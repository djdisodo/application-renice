use std::{fs::File, io::{BufRead, BufReader}, path::Path};

fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();
    let executable = args.next().unwrap_or("application-renice".to_string());
    if let Some(path) = args.next() {
        let restore = args.next() == Some("restore".to_string());
        process(Path::new(&path), restore)
    } else {
        println!("usage: {} [APPLICATIONS_DIR] [restore(optional)]", executable);
        Ok(())
    }
}

fn process(path: &Path, restore: bool) -> Result<(), std::io::Error> {
    if path.is_dir() {
        let dir = std::fs::read_dir(path)?;
        for dir_entry in dir {
            let dir_entry = dir_entry?;
            process(dir_entry.path().as_path(), restore)?;
        }
        Ok(())
    } else if path.file_name().map(|x| x.to_str().map(|x| x.ends_with(".desktop"))) == Some(Some(true)){
        let file = File::open(path)?;
        let buf_read: BufReader<_> = BufReader::new(file);
        let lines = buf_read.lines();
        let mut new_file_lines = Vec::with_capacity(lines.size_hint().0 + 1);
        for line in lines {
            let line = line?;
            if line.starts_with("###application-renice Exec") {
                if !restore {
                    return Ok(());
                }
                let header_len = "###application-renice ".len();
                new_file_lines.push(line[header_len..].to_string());
                continue;
            } else if !line.starts_with("Exec") {
                new_file_lines.push(line);
                continue;
            } else if restore {
                continue;
            }
            let mut line_splitted = line.split('=');
            if let [_, Some(value)] = [(); 2].map(|_| line_splitted.next()) {
                new_file_lines.push(format!("###application-renice Exec={}", value));
                new_file_lines.push(format!("Exec=nice -n 1 {}", value));
            }
        }
        let new_file_data = new_file_lines.join("\n");

        std::fs::write(path, new_file_data.as_bytes())?;

        Ok(())
    } else {
        Ok(())
    }

}
