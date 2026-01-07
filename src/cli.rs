use st3215::ST3215;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cogni-robot - Initialisation des servomoteurs ===");
    println!("Appuyez sur Ctrl+C pour quitter\n");

    let mut last_servos: Vec<u8> = Vec::new();
    let mut servo_connected = false;

    loop {
        // Tentative de connexion/reconnexion à la carte
        match ST3215::new("ACM03") {
            Ok(servo) => {
                if !servo_connected {
                    println!("Carte de contrôle détectée sur ACM03");
                    servo_connected = true;
                }

                // Récupérer la liste des servomoteurs connectés
                let servos = servo.list_servos();

                // Détecter les changements
                if servos != last_servos {
                    if servos.is_empty() {
                        println!("/!\\ Aucun servomoteur détecté");
                    } else {
                        println!("Servomoteurs connectés: {:?} (Total: {})", servos, servos.len());
                        
                        // Proposer l'initialisation si un seul servo est connecté
                        if servos.len() == 1 {
                            println!("\nUn seul servomoteur détecté (ID: {})", servos[0]);
                            println!("Voulez-vous changer son ID ? (o/n)");
                            
                            let mut input = String::new();
                            if std::io::stdin().read_line(&mut input).is_ok() {
                                if input.trim().to_lowercase() == "o" {
                                    println!("Entrez la nouvelle ID (0-253):");
                                    let mut id_input = String::new();
                                    if std::io::stdin().read_line(&mut id_input).is_ok() {
                                        if let Ok(new_id) = id_input.trim().parse::<u8>() {
                                            match servo.change_id(servos[0], new_id) {
                                                Ok(_) => println!("✓ ID changée avec succès: {} → {}\n", servos[0], new_id),
                                                Err(e) => println!("✗ Erreur: {}\n", e),
                                            }
                                        }
                                    }
                                }
                            }
                        } else if servos.len() > 1 {
                            println!("/!\\ Plusieurs servomoteurs détectés. Connectez-en un seul pour changer l'ID.");
                        }
                    }
                    
                    last_servos = servos;
                }
            }
            Err(_) => {
                if servo_connected {
                    println!("/!\\ Carte de contrôle déconnectée");
                    servo_connected = false;
                    last_servos.clear();
                }
            }
        }

        // Attendre avant la prochaine détection
        thread::sleep(Duration::from_millis(1000));
    }
}
