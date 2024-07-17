function postButton(destination, optionBox) {
    if (!destination) {
        return;
    }
    
    // Request to start a new session from the server
    // Wait for 2xx response
    // Then redirect to the session page
    
    // enable loading spinner
    let loadingAni = document.getElementById('loading');
    loadingAni.style.display = 'block';

    fetch(destination, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        }
    })
    .then(response => response.text())
    .then(data => {
        document.getElementById(optionBox).innerHTML = data
    })
    .finally(() => {
        loadingAni.style.display = 'none';  
    });
}