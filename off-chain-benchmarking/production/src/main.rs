extern crate rand7;
extern crate rand8;
extern crate ed25519_dalek;
extern crate bls_signatures;

use rand7::rngs::OsRng;
use rand8::Rng;
use ed25519_dalek::Keypair;
use ed25519_dalek::{Signature, Signer};
use bls_signatures::*;
use bls_signatures::Signature as bls_Signatures;
use bls_signatures::PrivateKey as bls_PrivateKey;
use std::time::{Duration, Instant};

fn main() {
    let rng = &mut rand8::thread_rng();
    let msg: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    let private_key = PrivateKey::generate(rng);
    let private = private_key.as_bytes();
    let public_key = vec![private_key.public_key()];
    let sig = private_key.sign(&msg);
    println!("{:?}", bls_PrivateKey::from_bytes(&private).unwrap());
    // measure_single_message_verious_length();
}

pub fn measure_multiple_different_messages() {
    let mut time: Vec<(i32, u128, u128)> = Vec::new(); 
    let mut num_of_msg: i32 = 1;
    while num_of_msg <= 2048{
        
        let rng = &mut rand8::thread_rng();
        let msgs: Vec<Vec<u8>> = (0..num_of_msg).map(|_| (0..64).map(|_| rng.gen()).collect()).collect();

        // // eddsa
        let mut csprng = OsRng{};
        let keypairs: Vec<_> = (0..num_of_msg).map(|_| Keypair::generate(&mut csprng)).collect();
        let mut j: usize = 0;
        let eddsa_sigs = msgs.iter().zip(&keypairs).map(|(msg, keypair)| keypair.sign(msg)).collect::<Vec<Signature>>();
        let eddsa_start = Instant::now();
        while j < num_of_msg.try_into().unwrap(){
            assert!(keypairs[j].verify(&msgs[j], &eddsa_sigs[j]).is_ok());
            j += 1;
        }
        let eddsa_duration = eddsa_start.elapsed().as_micros();

        // bls 
        let private_keys: Vec<_> = (0..num_of_msg).map(|_| PrivateKey::generate(rng)).collect();
        let public_keys = private_keys.iter().map(|private_key| private_key.public_key()).collect::<Vec<_>>();
        let sigs = msgs.iter().zip(&private_keys).map(|(msg, pk)| pk.sign(msg)).collect::<Vec<bls_Signatures>>();
        let aggregated_sig = aggregate(&sigs[..]).unwrap();
        let hashes = msgs.iter().map(|msg| hash(msg)).collect::<Vec<_>>();
        let bls_start = Instant::now();
        verify(&aggregated_sig, &hashes, &public_keys);
        let bls_duration = bls_start.elapsed().as_micros();

        time.push((num_of_msg, eddsa_duration, bls_duration));

        num_of_msg *= 2;
    }

    println!("{:?}", time);
}


pub fn measure_single_message_verious_length() {
    let num_of_iter = 1;
    let mut i = 1;
    let mut time: Vec<(i32, u128, u128)> = Vec::new(); 
    while i <= 100{
        let msg_len = 64*i;
        let rng = &mut rand8::thread_rng();
        let msg: Vec<u8> = (0..msg_len).map(|_| rng.gen()).collect();

        // eddsa
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let eddsa_start = Instant::now();
        let mut j = 0;
        while j < num_of_iter{
            let signature: Signature = keypair.sign(&msg);
            assert!(keypair.verify(&msg, &signature).is_ok());
            j += 1;
        }
        let eddsa_duration = eddsa_start.elapsed().as_micros() / num_of_iter;

        // bls 
        let private_key = PrivateKey::generate(rng);
        let public_key = vec![private_key.public_key()];

        let bls_start = Instant::now();
        let mut k = 0;
        while k < num_of_iter{
            let sig = private_key.sign(&msg);
            let hash = vec![hash(&msg)];
            verify(&sig, &hash, &public_key);
            k += 1;
        }
        
        let bls_duration = bls_start.elapsed().as_micros() / num_of_iter;
        time.push((msg_len, eddsa_duration, bls_duration));

        i += 1;
    }
    println!("{:?}", time);
}

