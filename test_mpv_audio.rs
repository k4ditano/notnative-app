use libmpv2::{Mpv, SetData};
use std::thread;
use std::time::Duration;

fn main() {
    unsafe {
        let locale = std::ffi::CString::new("LC_NUMERIC").unwrap();
        let c_locale = std::ffi::CString::new("C").unwrap();
        libc::setlocale(libc::LC_NUMERIC, c_locale.as_ptr());
    }

    let mpv = Mpv::new().expect("No se pudo crear MPV");
    
    // Configuración básica
    mpv.set_property("vid", "no").unwrap();
    mpv.set_property("video", "no").unwrap();
    mpv.set_property("ao", "pulse").unwrap();
    mpv.set_property("audio-device", "pulse/alsa_output.pci-0000_01_00.1.hdmi-stereo").unwrap();
    mpv.set_property("volume", 100).unwrap();
    mpv.set_property("mute", false).unwrap();
    
    println!("MPV inicializado");
    
    // URL de prueba - audio corto
    let test_url = "https://www2.cs.uic.edu/~i101/SoundFiles/BabyElephantWalk60.wav";
    
    println!("Cargando: {}", test_url);
    mpv.command("loadfile", &[test_url]).unwrap();
    
    println!("Asegurando que no está pausado...");
    mpv.set_property("pause", false).unwrap();
    
    // Esperar 10 segundos
    for i in 1..=10 {
        thread::sleep(Duration::from_secs(1));
        
        if let Ok(paused) = mpv.get_property::<bool>("pause") {
            println!("Segundo {}: pausado={}", i, paused);
        }
        
        if let Ok(time) = mpv.get_property::<f64>("time-pos") {
            println!("  Tiempo: {:.2}s", time);
        }
        
        if let Ok(vol) = mpv.get_property::<i64>("volume") {
            println!("  Volumen: {}%", vol);
        }
    }
    
    println!("Fin de la prueba");
}
