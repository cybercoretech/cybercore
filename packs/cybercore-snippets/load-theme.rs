//! Read the active Cybercore theme and any theme by name.
//! Cargo.toml: cybercore = "0.8"

fn main() {
    let schema = cybercore::schema::load();

    let active = schema.active_theme();
    println!("active theme: {} (bg #{}, accent #{})", schema.active, active.bg, active.acid_green);

    if let Some(palette) = schema.theme("cybercore-tech") {
        let family = schema.theme_family("cybercore-tech").unwrap_or("?");
        println!("cybercore-tech from {family}: cyan #{}", palette.cyan);
    }

    println!("{} themes available", schema.theme_names().count());
}
