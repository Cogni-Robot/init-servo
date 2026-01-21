use st3215::ST3215;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cogni-robot - Test de la bibliothèque ST3215 ===");

    // Initialisation de la connexion à la carte de contrôle
    let servo = ST3215::new("/dev/ttyACM0")?;

    // Récupérer la liste des servomoteurs connectés
    let servos = servo.list_servos();
    println!("Servomoteurs connectés: {:?} (Total: {})", servos, servos.len());

    Ok(())
}