#!/bin/bash

echo "Starting gtk3 at $BROADWAY_3_PORT $BROADWAY_3_SESSION, gtk4 at $BROADWAY_4_PORT $BROADWAY_4_SESSION"
/bin/su gtk-user -c "broadwayd -p $BROADWAY_3_PORT $BROADWAY_3_SESSION &"
/bin/su gtk-user -c "gtk4-broadwayd -p $BROADWAY_4_PORT $BROADWAY_4_SESSION &"

services=$(jq -r '.' /apps.json)
echo "Found: $services"
echo "Starting Watchdog..."
    while true; do
        for i in $(seq 0 $(($(echo $services | jq '. | length')-1)))
        do
        app=$(echo $services | jq -r ".[$i].app")
        args=$(echo $services | jq -r ".[$i].args")
        gtk_version=$(echo $services | jq -r ".[$i].gtk")
        if [ "$(ps -e |  head -n -2 | grep "${app}")" == '' ]
            then
                # If the service is not running, restart it
                echo "Service $app is not running, restarting..."
                if [ $gtk_version -eq 3 ]; then
                    export BROADWAY_DISPLAY="$BROADWAY_3_SESSION";
                else 
                    export BROADWAY_DISPLAY="$BROADWAY_4_SESSION";    
                fi
                /bin/su gtk-user -c "$app &" #GDK_BACKEND=broadway BROADWAY_DISPLAY=$BROADWAY_DISPLAY
            fi
        done
        sleep 30
    done
}
