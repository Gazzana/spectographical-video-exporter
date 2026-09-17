use std::process::Command;
use std::str;
use std::fs;
use std::path::{PathBuf};

struct Caminhos {
    sonograma : PathBuf,
    audio : PathBuf,
}

fn search(path: &str) -> Vec<Caminhos> {
    let mut caminhos: Vec<Caminhos> = Vec::new();

    // Primeira leitura
    let entradas = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return caminhos // retorna a lista vazia caso "path" não exista
    };

    for entrada in entradas.flatten() {
        let caminho_pasta = entrada.path();

        if caminho_pasta.is_dir() {
            let mut sonograma = None;
            let mut audio = None;

            // Busca dentro de cada subpasta
            if let Ok(arquivos) = fs::read_dir(&caminho_pasta) {
                for arquivo in arquivos.flatten() {
                    let caminho_arq = arquivo.path();

                    if caminho_arq.is_file() {
                        if let Some(ext) = caminho_arq.extension() {
                            if ext == "png" {
                                sonograma = Some(caminho_arq);
                            } else if ext == "mp3" {
                                audio = Some(caminho_arq);
                            }
                        }
                    }
                }
            }

            // se encontrou ambos, dai cria o sctruct caminhos
            if let (Some(png), Some(mp3)) = (sonograma, audio) {
                caminhos.push(Caminhos { sonograma: png, audio: mp3 });
            }
        }
    }

    return caminhos
}

fn main() {
    let paths = search("./samples");
    
    for i in paths {
        println!("Executando em:");
        println!("Áudio: {:?}", i.audio);
        println!("Imagem: {:?}", i.sonograma);

        generate_sonogram(i.audio, i.sonograma);

    }
}

fn generate_sonogram(audio_path: PathBuf, sonograma_path: PathBuf) {

    //                      ARRUMAR IMEDIATAMENTE
    
    let audio_path_str = &audio_path.display().to_string();
    let sonograma_path_str = &sonograma_path.display().to_string();
    
    let output = audio_path.display().to_string()+".mp4";

    let ffprobe_output = Command::new("ffprobe")
    .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            audio_path_str,
        ])
        .output()
        .expect("Falha ao executar o ffprobe");

    let duracao = str::from_utf8(&ffprobe_output.stdout).unwrap().trim();

    // montar o filtro complexo com a duração real
    // ex: "[1:v][2:v]overlay=x='(W/125.4)*t':y=0:shortest=1[video]"
    let filtro = format!("[1:v][2:v]overlay=x='(W/{})*t':y=0:shortest=1[video]", duracao);

    // executar o ffmpeg
    let status = Command::new("ffmpeg")
        .args(&[
            "-i", audio_path_str,
            "-loop", "1", "-framerate", "24", "-i", sonograma_path_str,
            "-f", "lavfi", "-i", "color=c=red:s=3x1080", // agulha vermelha de 3px
            "-filter_complex", &filtro,
            "-map", "[video]",
            "-map", "0:a",
            "-c:v", "libx264",
            "-preset", "fast", // Mude para "ultrafast" se quiser renderizar em segundos
            "-pix_fmt", "yuv420p",
            "-c:a", "copy",
            "-shortest",
            "-y", // Sobrescreve o arquivo de saída se existir
            &output,
        ])
        .status()
        .expect("Falha ao executar o ffmpeg");
    if status.success() {
        println!("Vídeo renderizado com sucesso!");
    } else {
        eprintln!("Erro na renderização.");
    }
}