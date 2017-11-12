URI parse based on nom composing parser library.

Provides simple parser that parses URI string  into structure optimized for processing - Query is parsed to HashTable, path to Path etc.

Consider this code:
```
extern crate hyper;
extern crate uri_parser;

use uri_parser::parse_uri;
use std::time::{Duration, SystemTime};

use hyper::Uri;
use std::str::FromStr;

fn dur_f64(d: Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 / 1e9
}

fn main() {
    let count = 1_000_000;
    let a_uri = "http://www.example.com/root/test?kulo=sak&kde=je&help=no&usi=yes#middle";
    let start = SystemTime::now();
    for _i in 0..count {
        let u = parse_uri(a_uri).unwrap();
        let d = u.query.unwrap();
        let h=d.get("help").unwrap();
        assert_eq!(*h, "no");
    }
    let dur = start.elapsed().unwrap();
    println!("{} loops of my parse_uri took {} secs", count, dur_f64(dur));
    let start = SystemTime::now();
    for _i in 0..count {
        let u = Uri::from_str(a_uri).unwrap();
        let q = u.query().unwrap();
        for qi in q.split("&") {
            let kv: Vec<_> = qi.split("=").collect();
            if kv[0] == "help" {
                let h = kv[1];
                assert_eq!(h, "no");
            }
        }
    }
    let dur = start.elapsed().unwrap();
    println!("{} loops of hyper from_str took {} secs", count, dur_f64(dur));
}
```

This library will perform better as query string is already in HashMap:
```
1000000 loops of my parse_uri took 0.900562534 secs
1000000 loops of hyper from_str took 1.136823832 secs
```

Limitations:
===========

1. Parses only absolute URIs
2. Will not parse IP6 host (yet)
3. No decoding of URL encoded strings (%hexa) -  because it's referring original string
4. Will not work well with malformed URI, only very basic parsing errors handling