fn main() -> Result<(), String> {
    println!("Initializing sdl2...");
    sdl2::init()?;
    println!("Ok");
    Ok(())
}
