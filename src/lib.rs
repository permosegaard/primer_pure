// http://www.codeproject.com/Articles/691200/Primality-test-algorithms-Prime-test-The-fastest-w
// https://github.com/danaj/Math-Prime-Util-GMP/blob/17b83d60a2f9bffe14c9116d2bde920e7bee46a0/gmp_main.c
// https://gist.github.com/jsanders/8739134
// https://rust-num.github.io/num/num/index.html

fn rough_root( number: &i64 ) -> i64 { return ( *number as f64 ).sqrt() as i64 + 1; }

fn divisible_by_x( number: &i64, x: &i64 ) -> bool { match *number % *x { 0 => return true, _ => return false } }
fn divisible_by_two( number: &i64 ) -> bool { return divisible_by_x( number, &2 ); } // test last bit??


pub enum PrimeTesters { Naive, SieveOfEratosthenes }

fn naive_primality( number: &i64 ) -> bool {
    match *number { 0 | 1 => return false, 2 => return true, _ => {} };

    // replace with augmented smart_increment
    if divisible_by_two( &number ) { return false; } 
    
    let root = rough_root( &number );
    
    let mut current = 3;
    while current <= root {

        match divisible_by_x( &number, &current ) {

            true => return false,
            false => current += 2 // should probably be replaced by smart increment

        }

    }
    
    return true;
}

fn sieve_of_eratosthenes_primality( number: &i64 ) -> bool {
    match *number { 0 | 1 => return false, 2 => return true, _ => {} };
    
    let mut current = 0;
    while current < *number {
    
        let mut multiplier = 0;
        while multiplier < *number {

            if current * multiplier == *number { return false; }
            else if current * multiplier > *number { break; }
            else { multiplier += 1; }

        }

        current += 1;

    }

    return true;
}

fn is_prime( number: &i64, tester: &PrimeTesters ) -> bool {
    match tester {
        &PrimeTesters::Naive => return naive_primality( number ),
        &PrimeTesters::SieveOfEratosthenes => return sieve_of_eratosthenes_primality( number )
    }
}

pub struct Primes { testing: i64, tester: PrimeTesters }
impl Primes {
    fn test( &mut self ) -> bool { return is_prime( &self.testing, &self.tester ); }
    fn smart_increment( &mut self ) {
        match self.testing { 0...2 => self.testing += 1, _ => self.testing += 2 }
    }
    fn increment( &mut self ) { self.smart_increment(); }
    
    pub fn get_testing( &mut self ) -> i64 { return self.testing; }
    pub fn set_testing( &mut self, testing: i64 ) { self.testing = testing; }
    pub fn test_and_increment( &mut self ) -> Option<i64> {
        match self.test() {
            true => { let prime = self.testing; self.increment(); return Some( prime ); },
            false => { self.increment(); return None; }
        }
    }
}
impl Iterator for Primes {
    type Item = i64;
    fn next( &mut self ) -> Option<i64> {
        loop {
            match self.test_and_increment() {
                Some( prime ) => return Some( prime ),
                None => {}
            }
        }
    }
//  fn skip( &mut self ) blah { self.current = arg } // speed optimisation, set to absolute number?!
}

fn primes( tester: PrimeTesters ) -> Primes { return Primes { testing: 0, tester: tester }; }
pub fn primes_naive() -> Primes { return primes( PrimeTesters::Naive ); }
pub fn primes_sieve_of_eratosthenes() -> Primes { return primes( PrimeTesters::SieveOfEratosthenes ); }
