use std::fs::File;
use std::io::{BufWriter, Write};
use std::str::FromStr;
use std::{array, prelude::*};
use std::{env, usize};

#[derive(Debug)]
struct Mono {
    co: f32,
    pow: f32,
}
#[derive(Debug)]
enum Eq {
    Val(Mono),
    Add(Box<Eq>, Box<Eq>),
    Mul(Box<Eq>, Box<Eq>),
}

impl FromStr for Mono {
    // expects form ax^b
    type Err = std::num::ParseFloatError;

    fn from_str(mono: &str) -> Result<Self, Self::Err> {
        let a: f32;
        let b: f32;
        if mono.contains("x") {
            let a_length = mono.chars().position(|c| c == 'x').unwrap();
            if a_length == 0 {
                a = 1.0;
            } else {
                a = f32::from_str(&mono[0..a_length])?;
            }
            if mono.contains("x^") {
                b = f32::from_str(&mono[a_length + 2..])?;
            } else {
                b = 1.0;
            }
        } else {
            a = f32::from_str(&mono)?;
            b = 0.0;
        }

        Ok(Mono { co: a, pow: b })
    }
}

impl FromStr for Eq {
    type Err = std::num::ParseFloatError;
    fn from_str(equation: &str) -> Result<Self, Self::Err> {
        if equation.contains("+") {
            let add_pos = equation.chars().position(|c| c == '+').unwrap();
            return Ok(Eq::Add(
                Box::new(Eq::from_str(&equation[0..add_pos])?),
                Box::new(Eq::from_str(&equation[add_pos + 1..])?),
            ));
        } else if equation.contains("*") {
            let mul_pos = equation.chars().position(|c| c == '*').unwrap();
            let part1 = Box::new(Eq::from_str(&equation[0..mul_pos])?);
            let part2 = Box::new(Eq::from_str(&equation[mul_pos + 1..])?);
            return Ok(Eq::Mul(part1, part2));
        }

        match Mono::from_str(equation) {
            Ok(mono) => Ok(Eq::Val(mono)),
            Err(e) => Err(e),
        }
    }
}

impl Eq {
    fn eval(&self, x: f32) -> f32 {
        match self {
            Eq::Add(e1, e2) => e1.eval(x) + e2.eval(x),
            Eq::Mul(e1, e2) => e1.eval(x) * e2.eval(x),
            Eq::Val(Mono { co: m, pow: n }) => m * (x.powf(*n)),
        }
    }
}

fn compute_table(n: usize) -> u32 {
    let mut c: u32 = u32::try_from(n).unwrap();
    for _ in 0..8 {
        if (c & 1) == 1 {
            c = 0xedb88320u32 ^ (c >> 1)
        } else {
            c = c >> 1
        }
    }
    return c;
}

fn chunker(c: u32) -> [u8; 4] {
    return [
        ((c >> 24) & 0xff) as u8,
        ((c >> 16) & 0xff) as u8,
        ((c >> 8) & 0xff) as u8,
        (c & 0xff) as u8,
    ];
}

fn lg2(num: usize) -> u8 {
    let mut counter = 0;
    while (num >> counter) > 1 {
        counter += 1;
    }
    return counter;
}
fn chk(num: u8, mode: u8) -> u8 {
    let num = 256 * u16::from(num);
    let mode = mode as u16;
    for i in 0..32 {
        if ((num | mode | (i)) % 31) == 0 {
            println!("check {:b}, {:X}", i << 3, num);
            return (i) as u8;
        }
    }
    return 0;
}
// fn huffman(data: &[u8]) -> [u8] {}

fn adlder32(data: &[u8]) -> u32 {
    let (mut a, mut b): (u32, u32) = (1, 0);
    for value in data {
        a = a + (*value as u32) % 65521;
        b = (b + a) % 65521;
    }
    return (b << 16) | a;
}

fn zlib_format(data: &[u8], length: u16) -> Vec<u8> {
    let window = length.next_power_of_two().max(256);
    let cmf: u8 = 0b0111_1000; // | ((lg2(window) - 8) << 4);
    let flg = chk(cmf, 0b000_0_0000);

    let mut img_stream: Vec<u8> = [cmf, flg, 0b000_0_0001].to_vec();
    let neg = length ^ 0xFFFF;
    img_stream.append(
        &mut ([
            length as u8,
            (length >> 8) as u8,
            neg as u8,
            (neg >> 8) as u8,
        ])
        .to_vec(),
    );
    img_stream.append(&mut Vec::from(data));
    img_stream.append(&mut Vec::from(chunker(adlder32(data))));
    return img_stream;
}

fn crc(data: Vec<u8>, table: &[u32; 256]) -> [u8; 4] {
    let mut c: u32 = 0xffffffffu32;
    for val in data {
        c = table[((c ^ (val as u32)) & 255) as usize] ^ ((c >> 8) & 0xFFFFFF);
    }
    c = c ^ 0xffffffff;
    println!("{:?}", c);
    return [
        ((c >> 24) & 0xff) as u8,
        ((c >> 16) & 0xff) as u8,
        ((c >> 8) & 0xff) as u8,
        (c & 0xff) as u8,
    ];
}

fn create_img(i: usize, row: usize, eq: &Eq) -> u8 {
    let val = (i % (row + 1));
    if val == 0 {
        return 0;
    } else {
        let x = ((val - 1) as f32);
        let y = ((row - ((i - val) / row)) as f32);
        if f32::powi(y - eq.eval(x), 2) <= 1.0 {
            return 0xFF;
        }
        return 0x00;
    }
}

fn create_graph(eq: Eq) -> std::io::Result<()> {
    const WIDTH_INT: usize = 128;
    const HIGHT_INT: usize = 128;
    let (width, hight): ([u8; 4], [u8; 4]) = (chunker(WIDTH_INT as u32), chunker(HIGHT_INT as u32));

    let crc_table: [u32; 256] = std::array::from_fn(|i| compute_table(i));
    let mut file = File::create("graph.png").unwrap();
    let header: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    let ihdr_length: [u8; 4] = [0, 0, 0, 13];
    let ihdr: [u8; 17] = [
        0x49, 0x48, 0x44, 0x52, width[0], width[1], width[2], width[3], hight[0], hight[1],
        hight[2], hight[3], 8, 0, 0, 0, 0,
    ];
    let ihdr_crc = crc(ihdr.to_vec(), &crc_table);
    file.write(&header)?;
    file.write(&ihdr_length)?;
    file.write(&ihdr)?;
    file.write(&ihdr_crc)?;
    let iend: [u8; 4] = [0x49, 0x45, 0x4E, 0x44];
    let iend_crc: [u8; 4] = crc(iend.to_vec(), &crc_table);
    let function: [u8; (WIDTH_INT + 1) * HIGHT_INT] =
        array::from_fn(|i| create_img(i, WIDTH_INT, &eq));
    let mut idat = vec![0x49, 0x44, 0x41, 0x54];
    idat.append(&mut zlib_format(
        &function,
        u16::try_from((WIDTH_INT + 1) * HIGHT_INT).unwrap(),
    ));
    let idat_crc = crc(idat.clone(), &crc_table);
    let length = chunker(u32::try_from(idat.len()).unwrap() - 4);
    println!("{:?}", length);
    let end_length = [0, 0, 0, 0];
    file.write(&length)?;
    for val in idat {
        file.write(&[val])?;
    }
    file.write(&idat_crc)?;
    file.write(&end_length)?;
    file.write(&iend)?;
    file.write(&iend_crc)?;
    file.flush()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("Please Provide Equation")
    } else {
        let equation = &args[1];
        match Eq::from_str(&equation) {
            Ok(eq) => {
                println!("Parsed: {:?}", eq);
                println!("Val: {:?}", eq.eval(0.0));

                let _ = create_graph(eq);
            }
            Err(e) => println!("Failed: {:?}", e),
        }
    }
}
