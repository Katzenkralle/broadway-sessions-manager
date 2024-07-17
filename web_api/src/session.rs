use db_controller::DbConn;
use db_controller::service_managment::modules::ActiveSession;


use docker_api::models::NetworkingConfig;
use rocket::serde::Serialize;
use rocket::response::status;
use rocket::State;
use rocket_dyn_templates::{context, Template}; 

use crate::DockerSettings;
use docker_api::docker::Docker;
use docker_api::opts::ContainerCreateOpts;
use docker_api::opts::PublishPort;
use docker_api::opts::HostPort;
use docker_api::models::EndpointSettings;
use docker_api::Id;
use docker_api::opts::ContainerRemoveOpts;

use crate::login::{SessionGuard, User};

use std::collections::HashMap;


pub const GTK3_SERVICE_ID: i32 = 2;
pub const GTK4_SERVICE_ID: i32 = 1;
pub const SELF_IP: &str = "127.0.0.1";

#[derive(Serialize)]
pub struct PortMap {
    pub gtk3: i32,
    pub gtk4: i32,
    pub ip: String
}


pub fn find_gtk_ports(active_sessions: &Vec<ActiveSession>) -> PortMap {
    let mut gtk_ports = PortMap {gtk3: 0, gtk4: 0, ip: SELF_IP.to_string()};
    active_sessions.iter().for_each(|session| {
        if session.service_id == GTK3_SERVICE_ID {
            gtk_ports.gtk3 = session.port.unwrap();
            gtk_ports.ip = session.container_ip.clone().unwrap_or(String::from(SELF_IP))
        }
        if session.service_id == GTK4_SERVICE_ID {
            gtk_ports.gtk4 = session.port.unwrap();
            gtk_ports.ip = session.container_ip.clone().unwrap_or(String::from(SELF_IP))
        }
    });
    gtk_ports
}

#[derive(Serialize)]
pub struct GtkBoxOptions {
    session_exists: bool,
    status_msg: String,
    gtk3_port: Option<i32>,
    gtk4_port: Option<i32>,
    container_ip: Option<String>,
}

impl GtkBoxOptions {
    pub fn find_status (active_sessions: Option<Vec<ActiveSession>>) -> GtkBoxOptions {
        match active_sessions {
            Some(sessions) => {
                let gtk_ports = find_gtk_ports(&sessions);
                return GtkBoxOptions {
                    session_exists: true,
                    status_msg: "Active session found".to_string(),
                    gtk3_port: Some(gtk_ports.gtk3),
                    gtk4_port: Some(gtk_ports.gtk4),
                    container_ip: Some(gtk_ports.ip),
                }
            },
            None => return GtkBoxOptions {
                session_exists: false,
                status_msg: "No active sessions".to_string(),
                gtk3_port: None,
                gtk4_port: None,
                container_ip: None
            },
        };
    }

    pub fn just_status(msg: &str) -> GtkBoxOptions {
        return GtkBoxOptions {
            session_exists: false,
            status_msg: msg.to_string(),
            gtk3_port: None,
            gtk4_port: None,
            container_ip: None
        }
    }
    pub fn build_template(self) -> Template {
        Template::render("partials/broadwayBoxOptions", context! {
            options: self
        })
    }
    
}

pub async fn status(db_conn: &State<DbConn>, user: &User, filter: Option<Vec<i32>>)
 -> Option<Vec<ActiveSession>> {
    
    if let Ok(mut active_sessions) = db_conn.get_sessions_by_user(&user.username) {
        active_sessions.retain(|s| 
            filter == None || 
            filter.as_ref().unwrap().contains(&s.service_id));
        if active_sessions.len() > 0 {
            return Some(active_sessions);
        }
    }
    None
}


#[post("/start/gtk", format = "json")]
pub async fn start(db_conn: &State<DbConn>, docker: &State<Docker>, docker_settings: &State<DockerSettings>, user: User) -> Result<status::Accepted<Template>, status::Conflict<Template>> {

    if let Some(existing_session) = status(db_conn, &user, Some(vec![GTK3_SERVICE_ID, GTK4_SERVICE_ID])).await {
        let mut msg: GtkBoxOptions = GtkBoxOptions::find_status(Some(existing_session));
        msg.status_msg = "Session already exists".to_string();
        return Ok(status::Accepted(msg.build_template()));
    }
    
    let mut errors: String = String::new();
    
    // Environment prepared
    // Regestrating in DB
    let (gtk3, gtk4) = match (db_conn.register_session(&user.username, GTK3_SERVICE_ID, true),
                                                           db_conn.register_session(&user.username, GTK4_SERVICE_ID, true)) {
        (Ok(gtk3), Ok(gtk4)) => (gtk3, gtk4),
        _ => return Err(status::Conflict(GtkBoxOptions::just_status("Faild to register session").build_template()))
    }; 


    // https://docs.docker.com/engine/api/v1.46/#tag/Container/operation/ContainerCreate
    // https://github.com/vv9k/docker-api-rs/blob/master/src/opts/container.rs
    let mut build_ops = ContainerCreateOpts::builder()
    .image(&docker_settings.vroot_image)
    .name(format!("vroot-{}-{}", gtk3.port.unwrap_or(0), gtk4.port.unwrap_or(0)))
    .env(vec![format!("BROADWAY_3_PORT={}", gtk3.port.unwrap_or(0)), 
        format!("BROADWAY_4_PORT={}", gtk4.port.unwrap_or(0))])
    .volumes(vec![format!("{}/someone:/home/gtk-user", &docker_settings.shared_volume)]);

    if let Some(common_network) = docker_settings.common_network.as_ref() {
        build_ops = build_ops.network_config(NetworkingConfig {
            endpoints_config: Some(HashMap::from([
                (
                    common_network.clone(),
                    EndpointSettings {
                        aliases: None,
                        driver_opts: None,
                        endpoint_id: None,
                        gateway: None,
                        global_i_pv_6_address: None,
                        global_i_pv_6_prefix_len: None,
                        ip_address: None,
                        ipam_config: None,
                        ip_prefix_len: None,
                        i_pv_6_gateway: None,
                        links: None,
                        mac_address: None,
                        network_id: None,
                    },
                ),
            ]))
        });
    } else {
        build_ops = build_ops.expose(PublishPort::tcp(gtk3.port.unwrap_or(0) as u32), HostPort::new(gtk3.port.unwrap_or(0) as u32))
            .expose(PublishPort::tcp(gtk4.port.unwrap_or(0) as u32), HostPort::new(gtk4.port.unwrap_or(0) as u32))
    }
    let final_ops = build_ops.build();


    
    let container_ip = match docker.containers().create(&final_ops).await {
        Ok(container) => {
            if container.start().await.is_err() {
                errors.push_str("; Failed to start container");
            }
            let container_id = container.id();
            let container_ip = match container.inspect().await {
                Ok(c) => {
                    let networks = match c.network_settings {
                        Some(n) => match n.networks {
                            Some(nn) => nn,
                            None => HashMap::new()
                        },
                        None => HashMap::new()
                    };
                    networks.get(&docker_settings.common_network.clone().unwrap())
                        .and_then(|n| n.ip_address.clone())
                        .unwrap_or_else(|| String::from("127.0.0.1"))
                },
                _ => String::from("127.0.0.1")
            };
            
            match (db_conn.add_docker_id(gtk3.id, container_id.as_ref(), Some(&container_ip)), 
                db_conn.add_docker_id(gtk4.id, container_id.as_ref(), Some(&container_ip))) {
                (Ok(()), Ok(())) => container_ip,
                _ => panic!("Failed to add docker_id {} to session {} and {}. This can not be handled!",
                     container_id, gtk3.id, gtk4.id),
            }
        },

        Err(err) => { println!("{:?}", err);
            let _ = db_conn.remove_session(user.username.as_str(), GTK3_SERVICE_ID);
            let _ = db_conn.remove_session(user.username.as_str(), GTK4_SERVICE_ID);
            return Err(status::Conflict(GtkBoxOptions::just_status("Failed to create container").build_template()))
        },
    };

   

    return Ok(status::Accepted(GtkBoxOptions {
        status_msg: format!("Session created {}", errors).to_string(),
        session_exists: true,
        gtk3_port: gtk3.port,
        gtk4_port: gtk4.port,
        container_ip: Some(container_ip.clone()),
        }.build_template()));
}

#[post("/end/gtk")]
pub async fn end(db_conn: &State<DbConn>, docker: &State<Docker>, user: User) -> Result<status::Accepted<Template>, status::Conflict<Template>> {
    
    let active_sessions = match status(db_conn, &user, Some(vec![GTK3_SERVICE_ID, GTK4_SERVICE_ID])).await {
        Some(s) => s,
        None => return Err(status::Conflict(GtkBoxOptions::just_status("Could not get active Sessions").build_template()))
    };
    
    let mut errors: String = String::new(); 

    if let None = active_sessions.get(0) {
        errors = "No active sessions; ".to_string();
    }

    let _username = user.username.clone();  


    let kill_ops = ContainerRemoveOpts::builder()
        .force(true)
        .volumes(true)
        .build();

    if active_sessions[0].docker_id != None {
        match docker.containers().get(Id::from(active_sessions[0].docker_id.as_ref().unwrap()))
            .remove(&kill_ops).await {
                Ok(_) => (),
                Err(docker_error) => errors = format!("{} Docker could not kill container: {:?};", errors, docker_error)
            }
    }
    
    let _ = db_conn.remove_session(&user.username, GTK3_SERVICE_ID);
    let _ = db_conn.remove_session(&user.username, GTK4_SERVICE_ID);
        
    
    return Ok(status::Accepted(GtkBoxOptions::just_status(
        if errors.is_empty() { "Stopped and Saved env" } else { &errors }
    ).build_template()));
}

#[get("/session-auth")]
pub fn handover(_seession: SessionGuard) -> status::Accepted<()> {
    // All Authorisation is done by the session guard
    return status::Accepted(());
}