fn main() {
    // Tell cargo to rerun if migrations change
    println!("cargo:rerun-if-changed=migrations");
    
    // Tell cargo to rerun if .env changes
    println!("cargo:rerun-if-changed=.env");
}