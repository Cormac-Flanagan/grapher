use std::env;
use std::str::FromStr;

#[derive(Debug)]
struct Mono {
    co: i32,
    pow: i32,
}
#[derive(Debug)]
enum Eq {
    Val(Mono),
    Add(Box<Eq>, Box<Eq>),
    Mul(Box<Eq>, Box<Eq>),
}

impl FromStr for Mono {
    // expects form ax^b
    type Err = std::num::ParseIntError;

    fn from_str(mono: &str) -> Result<Self, Self::Err> {
        let (a, b);
        if mono.contains("x") {
            let a_length = mono.chars().position(|c| c == 'x').unwrap();
            if a_length == 0 {
                a = 1;
            } else {
                a = i32::from_str_radix(&mono[0..a_length], 10)?;
            }
            if mono.contains("x^") {
                b = i32::from_str_radix(&mono[a_length+2..], 10)?;
            } else {
                b = 1;
            }
        } else {
            a = i32::from_str_radix(&mono, 10)?;
            b = 0;
        }

        Ok( Mono {co: a, pow: b} )
    }
}

impl FromStr for Eq {
    type Err = std::num::ParseIntError;
    fn from_str(equation: &str) -> Result<Self, Self::Err> {
        if equation.contains("*") {
            let mul_pos = equation.chars().position(|c| c == '*').unwrap();
            let part1 = Box::new(Eq::from_str(&equation[0..mul_pos])?);
            let part2 = Box::new(Eq::from_str(&equation[mul_pos+1..])?);
            return Ok(Eq::Mul(part1, part2))
        } else if equation.contains("+") {
            let add_pos = equation.chars().position(|c| c == '+').unwrap();
            return Ok(Eq::Add(Box::new(Eq::from_str(&equation[0..add_pos])?), Box::new(Eq::from_str(&equation[add_pos+1..])?)))
        }

        match Mono::from_str(equation) {
            Ok(mono) => Ok(Eq::Val(mono)),
            Err(e) => Err(e),
        } 
        

    }
}



fn main() {
    let args: Vec<String> = env::args().collect();
    let equation = &args[1];
    match Eq::from_str(&equation) {
        Ok(eq) => println!("Parsed: {:?}", eq),
        Err(e) => println!("Failed: {:?}", e),
    }
}
