use std::env;
use std::error;
use std::fs;
use std::io;

enum Options {
    All,
    Lines,
    Words,
    Bytes,
    Chars,
}

impl Options {
    fn from_str(s: &str) -> Options {
        match s {
            "l" => Options::Lines,
            "w" => Options::Words,
            "c" => Options::Bytes,
            "m" => Options::Chars,
            _ => Options::All,
        }
    }
}

// TODO: fix incorrect counts with smaller buffers; likely due to splitting multi-byte characters
const BUF_LEN: usize = 1_024_000;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let options: Options;
    let prefix_flag = args.len() > 1 && &args[1][..1] == "-";

    let mut reader_buffered: Box<dyn io::BufRead>;
    let stdin = io::stdin().lock();

    match args.len() {
        1 => {
            options = Options::All;
            reader_buffered = Box::new(create_buffered_reader(BUF_LEN, stdin));
        },
        2 => {
            if prefix_flag {
                options = Options::All;
                reader_buffered = Box::new(create_buffered_reader(BUF_LEN, stdin));
            } else {
                options = Options::All;
                let file = &args[1];
                reader_buffered = Box::new(create_buffered_reader(BUF_LEN, fs::File::open(file)?));
            }
        },
        3 => {
            options = Options::from_str(&args[1][1..]);
            let file = &args[2];
            reader_buffered = Box::new(create_buffered_reader(BUF_LEN, fs::File::open(file)?));
        },
        _ => return Err(From::from("invalid arguments")),
    }

    match process(&mut reader_buffered, options) {
        Ok(_) => {},
        Err(e) => eprintln!("{}", e),
    }

    if (args.len() == 2 && !prefix_flag) || args.len() == 3 {
        println!(" {}", &args[args.len() - 1]);
    } else {
        println!();
    }

    Ok(())
}

fn process(
    reader: &mut Box<dyn io::BufRead>,
    options: Options,
) -> Result<(), Box<dyn error::Error>> {
    let mut lines = 0;
    let mut words = 0;
    let mut bytes = 0;
    let mut chars = 0;

    let mut buf = [0; BUF_LEN];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let slice = std::str::from_utf8(&buf[..n])?;

        match options {
            Options::All => {
                bytes += n;
                lines += count_lines(slice);
                words += count_words(slice);
                chars += count_chars(slice);
            }
            Options::Lines => lines += count_lines(slice),
            Options::Words => words += count_words(slice),
            Options::Bytes => bytes += n,
            Options::Chars => chars += count_chars(slice),
        }
    }

    match options {
        Options::All => {
            print!("{:>8}{:>8}{:>8}{:>8}", lines, words, bytes, chars);
        },
        Options::Lines => {
            print!("{:>8}", lines);
        },
        Options::Words => {
            print!("{:>8}", words);
        },
        Options::Bytes => {
            print!("{:>8}", bytes);
        },
        Options::Chars => {
            print!("{:>8}", chars);
        },
    }

    Ok(())
}

fn count_lines(s: &str) -> usize {
    s.lines().count()
}

fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

fn count_chars(s: &str) -> usize {
    s.chars().count()
}

fn create_buffered_reader<R: io::Read>(capacity: usize, input: R) -> impl io::BufRead {
    io::BufReader::with_capacity(capacity, input)
}
