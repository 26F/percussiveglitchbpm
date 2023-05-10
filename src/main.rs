
/*

percussiveglitchbpm beats_per_minute glitch_every_beat probability_of_node_recurse probability_glitch

For example: percussiveglitchbpm input.wav 120 0.25 0.30

It doesn't work miracles. You will more than likely want to splice the result with the input
to keep the desired effect on a chunk and remove undesired artifacts.

Some software like MilkyTracker doesn't use standard definitions of bpm.



*/

use hound;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::env;


fn glitch_recursive(generator: &mut ThreadRng, vector: &mut Vec<usize>, recurseprob: f64, num_samples: usize, repetitions: usize) {

    let recurse: bool = generator.gen::<f64>() % 1.0 < recurseprob;

    if !recurse || num_samples <= 1 {

        println!("recursive call base cases reached with samples: {num_samples} and repetitions: {repetitions}");
        let nsamples = num_samples as usize;
        let times = repetitions as usize;

        for _ in 0..times {

            vector.push(nsamples);

        }

        return;

    }

    glitch_recursive(generator, vector, recurseprob, num_samples / 2, repetitions * 2);
    glitch_recursive(generator, vector, recurseprob, num_samples / 2, repetitions * 2);

}


fn main() {

    let args: Vec<_> = env::args().collect();

    // 120 BPM with possible glitch every 8th note.
    // with a probability of 0.75 of glitching 
    // percusiveglitchbpm input.wav 120 8 0.75
    // 4 4 is presumed.

    if args.len() < 6 {

        println!("percussiveglitchbpm input.wav 120 beat [one in n chance of recursing node glitch] [one in n chance of initial glitch causing cascade]");
        return;

    }

    let mut generator = rand::thread_rng();

    let filename: &str = args[1].trim();
    let  beats_per_minute: usize = args[2].trim().parse().expect("Failed to read bpm");
    let glitchfreqnote: usize = args[3].trim().parse().expect("Failed to read glitch note");
    let recurse_probability: f64 = args[4].trim().parse().expect("Failed to read don't recurse 1 in n");
    let glitch_probability: f64 = args[5].trim().parse().expect("Failed to read don't glitch probability");

    println!("bpm: {beats_per_minute} glitch every: {glitchfreqnote} probability of not recursing on glitch node: {recurse_probability} probability of not glitching: {glitch_probability}");

    let mut wavreader = hound::WavReader::open(filename).expect("Failed to open wav file");
    let spec = wavreader.spec();
    //let bitsps = spec.bits_per_sample - 1;
    //let maxsampleampl: i32 = i32::pow(2 as i32, bitsps as u32) - 1;

    // two samples per channel
    let samples_per_second;

    if spec.channels == 2 {

        samples_per_second = spec.sample_rate * 2;

    } else {

        samples_per_second = spec.sample_rate;

    };
    
    //let _beats_per_second: f64 = beats_per_minute as f64 / 60.0f64;
    
    // seconds per beat
    let quarter_note: f64 = 60.0f64 / beats_per_minute as f64;
    let eighth_note: f64 = quarter_note / 2.0f64;
    let sixteenth_note: f64 = eighth_note / 2.0f64;
    let thirty_secondth_note: f64 = sixteenth_note / 2.0f64;
    let sixtyfourth_note: f64 = thirty_secondth_note / 2.0f64;
    let note128th: f64 = sixtyfourth_note / 2.0f64;
    let note256th: f64 = note128th / 2.0f64;
    let note512: f64 = note256th / 2.0f64;
    let note1024th: f64 = note512 / 2.0f64;

    let half_note:  f64 = quarter_note * 2.0f64;
    let whole_note: f64 = half_note * 2.0f64;

    let notes = [whole_note, half_note, quarter_note, eighth_note, sixtyfourth_note, 
                 thirty_secondth_note, sixtyfourth_note, note128th, note256th, note512, 
                 note1024th];

    let letglitchnoteidx: usize = f64::log2(glitchfreqnote as f64) as usize;

    assert!(letglitchnoteidx < notes.len());

    let every: f64 = notes[letglitchnoteidx];

    let mut data: Vec<i32> = Vec::new();

    // load samples for indexing
    let samples = wavreader.samples::<i32>();

    for sample in samples {
        let smpl = sample.unwrap();
        data.push(smpl);

    }

    // we have a copy for write to
    let mut mutated: Vec<i32> = data.clone();
    
    // beats where we might glitch given recurse_probability
    let num_samples = data.len() as f64;
    let everybeat: f64 = every * samples_per_second as f64;

    let num_targeted = (num_samples / everybeat) as usize;
    let mut wavwriter = hound::WavWriter::create("outf.wav", spec).expect("Failed to create output wav");

    let glitch_durations_in_samples = [half_note * samples_per_second as f64, quarter_note * samples_per_second as f64, eighth_note * samples_per_second as f64, sixteenth_note * samples_per_second as f64];

    // One glitch event actually consists of four glitches.
    let mut glitches: Vec<usize> = Vec::new();

    for beatidx in 0..num_targeted {

        if generator.gen::<f64>() % 1.0 >= glitch_probability {
            glitches.clear();
            continue;

        }

        let sidx = (everybeat as f64 * beatidx as f64) as usize;
        let mutateidx = sidx;

        glitches.clear();

        let glitch_duration_samples: usize = glitch_durations_in_samples[generator.gen::<usize>() % glitch_durations_in_samples.len()] as usize;

        // recurse_probability not glitch
        println!("initial call to recursive");
        glitch_recursive(&mut generator, &mut glitches, recurse_probability, glitch_duration_samples, 1);

        let mut mutation_index = mutateidx;

        let mut testcount = 0;

        let mut kill = false;

        for nglitched_samples in &glitches {

            for glxsidx in 0..*nglitched_samples {

                if mutation_index + mutateidx < mutated.len() {

                    mutated[mutateidx + mutation_index] = data[mutateidx + glxsidx];

                    mutation_index += 1;

                    testcount += 1;

                    if testcount > glitch_duration_samples {

                        kill = true;
                        break;

                    }

                }

            }
            
            if kill {

                break;

            }

        }

    }

    for m_sample in mutated {

        wavwriter.write_sample(m_sample).expect("Failed to write sample");

    }

    wavwriter.finalize().unwrap();

}



