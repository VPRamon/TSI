// Test tiberius authentication methods
use tiberius::{AuthMethod, Config};

fn main() {
    // Check what auth methods are available
    let config = Config::new();
    
    // Available methods based on tiberius documentation:
    // 1. AuthMethod::sql_server(username, password) - SQL Server auth
    // 2. AuthMethod::windows(username) - Windows integrated auth
    // 3. AuthMethod::None - No authentication
    
    // For Azure AD, tiberius 0.12 supports it through token-based auth
    // but requires getting a token from Azure first
    
    println!("Checking tiberius auth methods...");
}
