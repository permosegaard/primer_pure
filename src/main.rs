// https://github.com/cyderize/rust-websocket/blob/master/examples/client.rs
extern crate time;
extern crate hyper;
//extern crate serde;
//extern crate serde_json;
extern crate rustc_serialize;

extern crate primer;

#[derive( Clone )]
struct PrimeStats { total_tests: i64, tests_history: Vec<PrimeStatsTimestamp>, last_prime: i64, total_primes: i64, primes_history: Vec<PrimeStatsTimestamp> }
impl PrimeStats {
    fn increment_tests( &mut self ) { self.total_tests += 1; }
    fn increment_primes( &mut self ) { self.total_primes += 1; }
    fn update_last_prime( &mut self, prime: i64 ) { self.last_prime = prime; }
    fn append_primes_history( &mut self, item: PrimeStatsTimestamp ) {
        if self.primes_history.len() >= 2 { self.primes_history.pop(); }
        self.primes_history.insert( 0, item );
    }
    fn append_tests_history( &mut self, item: PrimeStatsTimestamp ) {
        if self.tests_history.len() >= 2 { self.tests_history.pop(); }
        self.tests_history.insert( 0, item );
    }
    fn __x_in_seconds( &self, first: &PrimeStatsTimestamp, last: &PrimeStatsTimestamp ) -> i64 {
        return ( 1000000000 / last.timestamp.to( first.timestamp ).num_nanoseconds().unwrap() ) as i64;
    }
    fn tests_in_seconds( &self ) -> i64 {
        return self.__x_in_seconds( &self.tests_history[ 0 ], &self.tests_history[ self.tests_history.len() - 1 ] );
    }
    fn primes_in_seconds( &self ) -> i64 {
        return self.__x_in_seconds( &self.primes_history[ 0 ], &self.primes_history[ self.primes_history.len() - 1 ] );
    }
}

#[derive ( Clone )]
struct PrimeStatsTimestamp { _number: i64, timestamp: time::PreciseTime }

//static SHUTDOWN : bool = false;

fn update_stats( primestats: &mut PrimeStats, iterator: &mut primer::Primes ) {
    primestats.increment_tests();

    primestats.append_tests_history (
        PrimeStatsTimestamp { _number: iterator.get_testing(), timestamp: time::PreciseTime::now() }
    );
}

fn updates_stats_newprime( primestats: &mut PrimeStats, value: i64 ) {
    primestats.increment_primes();
    primestats.update_last_prime( value );

    primestats.append_primes_history(
        PrimeStatsTimestamp { _number: value, timestamp: time::PreciseTime::now() }
    );
}

fn find_primes( start_at: Option<i64>, end_at: Option<i64> ) {
    let mut primestats = PrimeStats { total_tests: 0, tests_history: Vec::new(), last_prime: 0, total_primes: 0, primes_history: Vec::new() };

    let mut iterator = primer::primes_naive();
    match start_at { Some( number ) => iterator.set_testing( number ), None => {} }
    
    let mut last_stats_dump = time::PreciseTime::now();
    
    loop {
        //if SHUTDOWN { println!( "last prime found: {}", &primestats.last_prime ); return; }

        match end_at {
            Some( number ) => { if iterator.get_testing() >= number { progress( primestats.clone() ); return; } },
            None => {}
        }

        update_stats( &mut primestats, &mut iterator );

        match iterator.test_and_increment() {
            Some( number ) => {
                //println!( "{}", primestats.last_prime ); // debug here
                updates_stats_newprime( &mut primestats, number )
            },
            None => {}
        }

        if last_stats_dump.to( time::PreciseTime::now() ) > time::Duration::seconds( 10 ) {
            progress( primestats.clone() );
            last_stats_dump = time::PreciseTime::now();
        }
    }
}

fn progress( primestats: PrimeStats ) { progress_print( primestats.clone() ); progress_send( primestats.clone() ); }

fn progress_print( primestats: PrimeStats ) {
    println!( "{}/s {}/s {} {} {}", &primestats.tests_in_seconds(), &primestats.primes_in_seconds(), &primestats.total_tests, &primestats.total_primes, &primestats.last_prime );
}

fn progress_send( primestats: PrimeStats ) {
    std::thread::spawn( move || {
        use rustc_serialize::base64::ToBase64; // required for trait imports

//        #[derive( RustcEncodable )]
        struct UpdateStats { total_tests: i64, total_primes: i64, speed_tests: i64, speed_primes: i64, last_prime: i64 }

// requires rust nightly :( swap out when nightly's in the repo
//        sender.send_message( ( serde_json::to_string(
//            &UpdateStats {
//                total_tests: primestats.total_tests,
//                total_primes: primestats.total_primes,
//                speed_tests: *&primestats.tests_in_seconds(),
//                speed_primes: *&primestats.primes_in_seconds(),
//                last_prime: primestats.last_prime,
//            }
//        ).unwrap().as_bytes() ).to_base64() );

        fn to_json( updatestats: UpdateStats ) -> String {
            return format!(
                "{{
                    total_tests: {t_t},
                    total_primes: {t_p},
                    speed_tests: {s_t},
                    speed_primes: {s_p},
                    last_prime: {l_p}
                }}",
                t_t = updatestats.total_tests,
                t_p = updatestats.total_primes,
                s_t = updatestats.speed_tests,
                s_p = updatestats.speed_primes,
                l_p = updatestats.last_prime
            );
        }

        let updatestats = UpdateStats {
            total_tests: primestats.total_tests,
            total_primes: primestats.total_primes,
            speed_tests: primestats.tests_in_seconds(),
            speed_primes: primestats.primes_in_seconds(),
            last_prime: primestats.last_prime,
        };

        let json = to_json( updatestats );
        let base64 = json.as_bytes().to_base64( rustc_serialize::base64::MIME );
        let body = base64.as_bytes();

        let client = hyper::Client::new();
        let request = client.post( "http://169.254.254.5/publish?id=primer" ).body( body );
        
        let _response = request.send();
    });
}

fn main() {
    let prime_thread = std::thread::spawn(
        move || find_primes(
            match std::env::args().len() { 2 => Some( std::env::args().nth( 1 ).unwrap().parse::<i64>().unwrap() ), _ => None },
            match std::env::args().len() { 3 => Some( std::env::args().nth( 2 ).unwrap().parse::<i64>().unwrap() ), _ => None }
        )
    );

    let _ = prime_thread.join();
}
