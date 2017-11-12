use nom::{IResult, digit, ErrorKind};
use std::str;
use std::path::Path;
use std::collections::HashMap;
use super::{URI,User};

named!(token<&[u8], &str>, map_res!(is_not!(":/?#[]@"), str::from_utf8));
named!(scheme <&[u8], &str>, map_res!(take_until!(":"), str::from_utf8));
named!(user <&[u8], User>, do_parse!(
    user: token >>
    password: opt!(do_parse!(
        tag!(":") >>
        password: token >>
        (password)
    )) >>
    tag!("@") >>
    (User{name:user, password:password})
));

named!(authority< &[u8], (Option<User>, &str, Option<u16>) >, 
do_parse!(
        tag!("//") >> 
        user: opt!(complete!(user)) >>
        host: token >>
        port: opt!(complete!(do_parse!(
            tag!(":") >>
            p: map_res!(digit, bytes_to_u16) >>
            (p)
        ))) >>
        (user, host, port)
        )
);
named!(path_token<&[u8], &str>, map_res!(is_not!(":?#[]"), str::from_utf8));
fn parse_path(i: &[u8]) -> IResult<&[u8], &Path> {
    if i.is_empty() || ! i[0] as char == '/' {
        return IResult::Error(ErrorKind::Custom(1));
    }
    path_token(i).map(|s| Path::new(s))
}

named!(query_token<&[u8], &str>, map_res!(is_not!("&=:#[]"), str::from_utf8));
named!(query_item<&[u8], (&str, &str)>, do_parse!(
    key: query_token >>
    char!('=') >>
    val: query_token >>
    (key,val)
));

named!(query<&[u8], HashMap<&str,&str> >, 
    map!(
    preceded!(
    tag!("?"),
    separated_list_complete!(char!('&'), query_item)
    ),
    |v: Vec<_>| v.into_iter().collect()
    )
);

named!(hash_token<&[u8], &str>, map_res!(is_not!(":#[]"), str::from_utf8));
named!(hash<&[u8], &str>, preceded!(
    tag!("#"),
    hash_token
));

named!(pub uri <&[u8], URI>, dbg!( do_parse!(
    scheme: scheme >>
    tag!(":") >>
    authority: opt!(authority) >>
    path: opt!(parse_path) >>
    query: opt!(complete!(query)) >>
    hash: opt!(complete!(hash)) >>
    
    ( match authority {
        Some(a) => URI {scheme, user:a.0, host:Some(a.1), port: a.2, path, query, hash},
        None => URI {scheme, user:None, host:None, port:None, path, query, hash}
    }
    )
)));

fn bytes_to_u16(b: &[u8]) -> Result<u16, String> {
    str::from_utf8(b)
        .map_err(|e| e.to_string())
        .and_then(|s| u16::from_str_radix(s, 10)
                .map_err(|e| e.to_string())
            )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let qs=b"?a=b&c=d";
        let d = query(qs).unwrap().1;
        assert_eq!(d.get("a"), Some(&"b"));
         assert_eq!(d.get("c"), Some(&"d"));  
    }

    #[test]
    fn test_conversion() {
        let n = b"1234";
        assert_eq!(bytes_to_u16(n), Ok(1234));
        
    }

    #[test]
    fn test_user() {
        let u="ivan@";
        assert_eq!(user(u.as_bytes()), IResult::Done("".as_bytes(), User{name:"ivan", password:None }));

        let u="ivan:heslo@";
        assert_eq!(user(u.as_bytes()), IResult::Done("".as_bytes(), User{name:"ivan", password:Some("heslo") }));

    }

    #[test]
    fn test_path() {
        assert_eq!(parse_path(b"/"), IResult::Done("".as_bytes(), Path::new("/")));
        assert!(parse_path(b"").is_err());
    }

    #[test]
    fn test_uri() {
        fn tst(u: &[u8], res: URI) -> () {
            use IResult::*;
            match uri(u) {
                Done(_, r) => assert_eq!(r, res),
                Error(e) => panic!("Parsing uri failed {:?}", e),
                Incomplete(i) => panic!("Incomplete parsing {:?}", i)
            }
        }
        let u=b"https://zderadicka.eu";
        tst(u, URI{scheme:"https", user:None, host:Some("zderadicka.eu"), port:None, path: None, query:None, hash:None});
        let u=b"https://zderadicka.eu:8080";
        tst(u, URI{scheme:"https", user:None, host:Some("zderadicka.eu"), port:Some(8080), path: None, query:None, hash:None});
        let u=b"https://ivan:secret@zderadicka.eu/";
        tst(u, URI{scheme: "https", user: Some(User{name:"ivan", password:Some("secret")}),
                host:Some("zderadicka.eu"), port:None, path: Some(Path::new("/")), query:None, hash:None});

        let u=b"https://ivan:secret@zderadicka.eu/home?q=hey#hash";
        let mut q = HashMap::new();
        q.insert("q", "hey");
        tst(u, URI{scheme: "https", user: Some(User{name:"ivan", password:Some("secret")}),
                host:Some("zderadicka.eu"), port:None, path: Some(Path::new("/home")), query:Some(q), hash:Some("hash")});
        
    }

    #[test]
    fn test_scheme() {
        let s = b"http:";
        assert_eq!(scheme(s), IResult::Done(":".as_bytes(), "http"));
    }

}
