//! Module to provide a light URI parser focused on easy access to
//! and inspection of URI components
//! 
//! # Examples
//! 
//! ```
//! use uri_parser::parse_uri;
//! 
//! let uri_string = "http://usak:kulisak@www.example.com:8080/root/test?kulo=sak&kde=je&help=no&usi=yes#middle";
//! let parsed_uri = parse_uri(uri_string).unwrap();
//! assert_eq!(parsed_uri.port, Some(8080));
//! assert_eq!(parsed_uri.host, Some("www.example.com"));
//! assert!(parsed_uri.user.is_some());
//! let d = parsed_uri.query.unwrap();
//! let h=d.get("help").unwrap();
//! assert_eq!(*h, "no");
//! ```
//! 
#[macro_use]
extern crate nom;

use nom::IResult;
use std::str::{self};
use std::path::Path;
use std::collections::HashMap;
use std::fmt::{self, Display};

pub mod parser;

/// Represents parsed URI structure
///  URI parts are scheme, user (struct with name and password), host, port
/// path (represented as std::path::Path), query (HashMap of key, value pairs)
/// and hash (fragment)
#[derive(Debug,PartialEq)]
pub struct URI<'a> {
    pub scheme: &'a str,
    pub user: Option<User<'a>>,
    pub host: Option<&'a str>,
    pub port: Option<u16>,
    pub path: Option<&'a Path>,
    pub query: Option<HashMap<&'a str, &'a str>>,
    pub hash: Option<&'a str>
}

impl <'a> Display for URI<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}:", self.scheme)?;
        if self.user.is_some() ||  self.host.is_some() {
            write!(f,"//")?;
        }
        if let  Some(User{name, password}) = self.user {
                write!(f,"{}",name)?;
                if let Some(pwd) = password {
                    write!(f, ":{}", pwd)?
                }
                write!(f,"@")?;
            }
        if let Some(host) = self.host {
            write!(f,"{}", host)?;
        }
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        if let Some(path) = self.path {
            write!(f, "{}", path.display())?;
        }
        if let Some(ref query) = self.query {
            write!(f,"?")?;
            let mut prev = false;
            for (key,val) in query.iter() {
                if prev {
                    write!(f,"&")?;
                } else {
                    prev = true;
                }
                write!(f,"{}={}", key,val)?;
            }
            
        }
        if let Some(hash) = self.hash {
            write!(f,"#{}", hash)?;
        }
        Ok(())
    }
}


// FromStr cannot be implemeneted as URI has lifetime param
// Could implement From, however does not make much sence, as one can get parsing error easily
// impl <'a> From<&'a str> for URI<'a> {
    
//     fn from(s: &'a str) -> Self {
//         parse_uri(s).unwrap()
//     }
// }

#[derive(Debug,PartialEq)]
pub struct User<'a> {
    name: &'a str,
    password: Option<&'a str>
}

/// Possible parsing errors
#[derive(Debug,PartialEq)]
pub enum Error {
    Parse(nom::Err),
    Incomplete,
    NotFullyParsed
}

impl fmt::Display for Error {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}: {:?}", <Error as std::error::Error>::description(self), self)
}
}

impl std::error::Error for Error {
fn description(&self) -> &str {
    "URI parsing error"
}
}

/// Parses URI from string or bytes slice
/// Returns Result with URI structure or parsing Error
pub fn parse_uri<T: AsRef<[u8]>+?Sized>(uri_string: &T) -> Result<URI,Error> {
    let b:&[u8] = uri_string.as_ref();
    match parser::uri(b) {
        IResult::Done(remaining, u) => if remaining.is_empty() {
                Ok(u)
            } else {
                Err(Error::NotFullyParsed)
            },
        IResult::Error(e) => Err(Error::Parse(e)),
        IResult::Incomplete(_) => Err(Error::Incomplete)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_uri() {
        //bytes works
        let u=b"http://nekde/nekdo";
        let res = parse_uri(u).unwrap();
        assert_eq!(res.path, Some(Path::new("/nekdo")));

        // str works
        let u="http://nekde/nekdo#nekdy";
        let res = parse_uri(u).unwrap();
        assert_eq!(res.hash, Some("nekdy"));

        // string works

        // str works
        let u="http://nekde:123/nekdo#nekdy".to_owned();
        let res = parse_uri(&u).unwrap();
        assert_eq!(res.port, Some(123));

    }

    #[test]
    fn test_display() {
        let u = "http://usak:kulisak@www.example.com:8080/root/test?kulo=sak&kde=je&help=no&usi=yes#middle";
        let us = parse_uri(u).unwrap();
        let u2 = format!("{}",us);
        println!("{}",us);
        let us2 = parse_uri(&u2).unwrap();
        assert_eq!(us, us2);

    }
}
