use crate::devices::discovery::Device;

// Base HTML template with CSS styling
pub fn base_template(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Dishwasher Monitor</title>
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 1rem;
        }}
        header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid #eee;
        }}
        h1, h2, h3 {{ color: #222; margin-top: 1.5em; }}
        a {{ color: #0066cc; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        .button {{
            display: inline-block;
            background: #0066cc;
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 4px;
            border: none;
            cursor: pointer;
            font-size: 1rem;
            text-decoration: none;
        }}
        .button:hover {{ background: #0055aa; text-decoration: none; }}
        .button.secondary {{ background: #f4f4f4; color: #333; border: 1px solid #ddd; }}
        .button.secondary:hover {{ background: #e8e8e8; }}
        .button.danger {{ background: #cc3300; }}
        .button.danger:hover {{ background: #aa2200; }}
        pre {{ background: #f4f4f4; padding: 1rem; border-radius: 4px; overflow-x: auto; }}
        .card {{
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1rem;
            margin-bottom: 1rem;
            background: white;
        }}
        .container {{ padding: 1rem 0; }}
        .form-group {{ margin-bottom: 1rem; }}
        label {{ display: block; margin-bottom: 0.5rem; font-weight: 500; }}
        input, select {{ width: 100%; padding: 0.5rem; font-size: 1rem; border: 1px solid #ddd; border-radius: 4px; }}
        .camera-list {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem; }}
        .camera-card {{ border: 1px solid #ddd; border-radius: 4px; padding: 1rem; }}
        .camera-card h3 {{ margin-top: 0; }}
        .actions {{ margin-top: 1rem; display: flex; gap: 0.5rem; }}
    </style>
</head>
<body>
    <header>
        <h1>Dishwasher Monitor</h1>
        <nav>
            <a href="/" class="button secondary">Home</a>
        </nav>
    </header>
    {}
    <footer style="margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #eee; text-align: center; font-size: 0.9rem; color: #666;">
        &copy; 2025 Dishwasher Monitor
    </footer>
</body>
</html>"#,
        title, content
    )
}

// Home page template
pub fn home_page() -> String {
    let content = r#"
    <div class="container">
        <h2>Welcome to Dishwasher Monitor</h2>
        <p>Monitor your dishwasher activity using your Nest cameras.</p>
        
        <div class="card">
            <h3>Get Started</h3>
            <p>Sign in with your Google account to connect your Nest cameras.</p>
            <a href="/auth/authorize" class="button">Sign in with Google</a>
        </div>
        
        <div class="card">
            <h3>How it Works</h3>
            <ol>
                <li>Sign in with your Google account</li>
                <li>Select which cameras to monitor</li>
                <li>We'll notify you when your dishwasher is running or finished</li>
            </ol>
        </div>
    </div>
    "#;
    
    base_template("Home", content)
}

// Camera selection page
pub fn camera_selection_page(user_id: &str, cameras: &[Device]) -> String {
    let mut camera_list = String::new();
    
    // Generate camera cards
    for camera in cameras {
        let location = camera.room_name.as_deref().unwrap_or("Unknown location");
        
        camera_list.push_str(&format!(
            r#"
            <div class="camera-card">
                <h3>{}</h3>
                <p><strong>Location:</strong> {}</p>
                <p><strong>ID:</strong> {}</p>
                <div class="actions">
                    <form action="/cameras/register" method="post">
                        <input type="hidden" name="user_id" value="{}">
                        <input type="hidden" name="device_id" value="{}">
                        <button type="submit" class="button">Add to Monitoring</button>
                    </form>
                </div>
            </div>
            "#,
            camera.display_name, location, camera.device_id, user_id, camera.device_id
        ));
    }
    
    // If no cameras found
    if cameras.is_empty() {
        camera_list = r#"
        <div class="card">
            <p>No cameras found. Make sure you have cameras configured in your Google Nest account.</p>
        </div>
        "#.to_string();
    }
    
    let content = format!(
        r#"
        <div class="container">
            <h2>Select Cameras to Monitor</h2>
            <p>Choose which cameras you want to use for monitoring your dishwasher.</p>
            
            <div class="camera-list">
                {}
            </div>
            
            <div style="margin-top: 2rem;">
                <a href="/dashboard" class="button secondary">Go to Dashboard</a>
            </div>
        </div>
        "#,
        camera_list
    );
    
    base_template("Select Cameras", &content)
}

// Dashboard page for managing cameras
pub fn dashboard_page(user_id: &str, registered_cameras: &[Device]) -> String {
    let mut camera_list = String::new();
    
    // Generate camera cards for registered cameras
    for camera in registered_cameras {
        let location = camera.room_name.as_deref().unwrap_or("Unknown location");
        
        camera_list.push_str(&format!(
            r#"
            <div class="camera-card">
                <h3>{}</h3>
                <p><strong>Location:</strong> {}</p>
                <p><strong>Status:</strong> Monitoring</p>
                <div class="actions">
                    <form action="/cameras/unregister" method="post">
                        <input type="hidden" name="user_id" value="{}">
                        <input type="hidden" name="device_id" value="{}">
                        <button type="submit" class="button danger">Remove</button>
                    </form>
                </div>
            </div>
            "#,
            camera.display_name, location, user_id, camera.device_id
        ));
    }
    
    // If no registered cameras
    if registered_cameras.is_empty() {
        camera_list = r#"
        <div class="card">
            <p>No cameras are currently being monitored. Add cameras from the selection page.</p>
        </div>
        "#.to_string();
    }
    
    let content = format!(
        r#"
        <div class="container">
            <h2>Your Dashboard</h2>
            <p>Manage your monitored cameras and view status.</p>
            
            <div style="margin-bottom: 1rem;">
                <a href="/cameras/select" class="button">Add More Cameras</a>
            </div>
            
            <h3>Currently Monitored Cameras</h3>
            <div class="camera-list">
                {}
            </div>
        </div>
        "#,
        camera_list
    );
    
    base_template("Dashboard", &content)
}

// Authorization success page
pub fn auth_success_page(user_id: &str) -> String {
    let content = format!(
        r#"
        <div class="container">
            <h2>Authorization Successful!</h2>
            <p>You've successfully authorized with your Google account.</p>
            
            <div class="card">
                <h3>Next Steps</h3>
                <p>You can now select which cameras you want to use for monitoring your dishwasher.</p>
                <a href="/cameras/select?user_id={}" class="button">Select Cameras</a>
            </div>
        </div>
        "#,
        user_id
    );
    
    base_template("Authorization Successful", &content)
}

// Error page
pub fn error_page(title: &str, message: &str) -> String {
    let content = format!(
        r#"
        <div class="container">
            <h2>{}</h2>
            <div class="card">
                <p>{}</p>
                <a href="/" class="button">Back to Home</a>
            </div>
        </div>
        "#,
        title, message
    );
    
    base_template(&format!("Error: {}", title), &content)
}