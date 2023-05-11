
/*

percussiveglitchbpm beats_per_minute glitch_every_beat probability_of_node_recurse probability_glitch

For example: percussiveglitchbpm input.wav 120 2 0.25 0.30
has bpm of 120, glitch probability on every half note, probability of node recurse on glitch of 0.25 
and probability of initial glitch (cause recurse cascade) of 0.30

You will more than likely want to splice the result with the input
to keep the desired effect on a chunk and remove undesired artifacts.

Some software like MilkyTracker doesn't use standard definitions of bpm.

You can also test that your assumption about tempo is correct:
percussiveglitchbpm input.wav 120 2 0.25 1.0
using a probability of always glitch puts (1.0) puts a click track over the song
so that you can test tempo.
*/

use hound;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::collections::VecDeque;
use std::env;


const DISPLAY_STR: &str = r#"
For example: percussiveglitchbpm input.wav 120 2 0.25 0.30
has bpm of 120, glitch probability on every half note, probability of node recurse on glitch of 0.25 
and probability of initial glitch (causing recurse) of 0.30

You will more than likely want to splice the result with the input
to keep the desired effect on a chunk and remove undesired artifacts.

Some software like MilkyTracker doesn't use standard definitions of bpm."#;


// fn glitch_recursive(generator: &mut ThreadRng, vector: &mut Vec<usize>, recurseprob: f64, num_samples: usize, repetitions: usize) {

//     let recurse: bool = generator.gen::<f64>() % 1.0 < recurseprob;

//     if !recurse || num_samples <= 1 {

//         let nsamples = num_samples as usize;
//         let times = repetitions as usize;

//         for _ in 0..times {

//             vector.push(nsamples);

//         }

//         return;

//     }

//     glitch_recursive(generator, vector, recurseprob, num_samples / 2, repetitions * 2);
//     glitch_recursive(generator, vector, recurseprob, num_samples / 2, repetitions * 2);

// }


fn glitch_iterative(generator: &mut ThreadRng, vector: &mut Vec<usize>, recurseprob: f64, num_samples: usize, repetitions: usize) {

    let mut stack: VecDeque<(f64, usize, usize)> = VecDeque::new();

    let kill_limit = 128; // prevent program crash from using too much memory

    stack.push_front((recurseprob, num_samples, repetitions));

    while let Some((recurseprob, num_samples, repetitions)) = stack.pop_front() {

        let recurse: bool = generator.gen::<f64>() % 1.0 < recurseprob;

        if !recurse || num_samples <= kill_limit {

            let nsamples = num_samples as usize;
            let times = repetitions as usize;

            for _ in 0..times {

                vector.push(nsamples);

            }

        } else {

            stack.push_front((recurseprob, num_samples / 2, repetitions * 2));
            stack.push_front((recurseprob, num_samples / 2, repetitions * 2));

        }

    }

}


fn main() {

    let args: Vec<_> = env::args().collect();

    if args.len() < 6 {

        println!("{}", DISPLAY_STR);
        return;

    }

    let mut generator = rand::thread_rng();

    let filename: &str = args[1].trim();
    let  beats_per_minute: usize = args[2].trim().parse().expect("Failed to read bpm");
    let glitchfreqnote: usize = args[3].trim().parse().expect("Failed to read glitch note");

    let recurse_probability: f64 = args[4].trim().parse().expect("Failed to read don't recurse 1 in n");
    let glitch_probability: f64 = args[5].trim().parse().expect("Failed to read don't glitch probability");

    let mut wavreader = hound::WavReader::open(filename).expect("Failed to open wav file");
    let spec = wavreader.spec();

    // two samples per channel
    let samples_per_second;

    // Since stereo uses two samples for one 2D sound sample point.
    if spec.channels == 2 {

        samples_per_second = spec.sample_rate * 2;

    } else {

        samples_per_second = spec.sample_rate;

    };
    
    // seconds per beat
    let quarter_note: f64 = 60.0f64 / beats_per_minute as f64;
    let eighth_note: f64 = quarter_note / 2.0f64;
    let sixteenth_note: f64 = eighth_note / 2.0f64;
    let thirty_secondth_note: f64 = sixteenth_note / 2.0f64;
    let sixtyfourth_note: f64 = thirty_secondth_note / 2.0f64;

    let half_note:  f64 = quarter_note * 2.0f64;
    let whole_note: f64 = half_note * 2.0f64;

    let notes = [whole_note, half_note, quarter_note, eighth_note, sixtyfourth_note, 
                 thirty_secondth_note, sixtyfourth_note];

    let glitchnoteidx: usize = f64::log2(glitchfreqnote as f64) as usize;

    assert!(glitchnoteidx < notes.len());

    let every: f64 = notes[glitchnoteidx];

    let mut data: Vec<i32> = Vec::new();

    // load samples for indexing
    let samples = wavreader.samples::<i32>();

    for sample in samples {
        let smpl = sample.unwrap();
        data.push(smpl);

    }

    // we have a copy for write to
    let mut mutated: Vec<i32> = data.clone();
    
    // beats where we might glitch given the probability of glitching initially.
    let num_samples = data.len() as f64;
    let everybeat: f64 = every * samples_per_second as f64;

    // num potential points of glitch (recurse start points)
    let num_targeted = (num_samples / everybeat) as usize;

    // output wave file
    let mut wavwriter = hound::WavWriter::create("outf.wav", spec).expect("Failed to create output wav");

    // How long is total glitch including its recurse nodes.
    let glitch_durations_in_samples = [whole_note * samples_per_second as f64, half_note * samples_per_second as f64, quarter_note * samples_per_second as f64, 
                                       eighth_note * samples_per_second as f64, sixteenth_note * samples_per_second as f64];

    // One glitch event consists of some number of "halvings" which are done recursively.
    let mut glitches: Vec<usize> = Vec::new();

    // glitch points are dictated by beatidx * everybeat
    for beatidx in 0..num_targeted {

        if generator.gen::<f64>() % 1.0 >= glitch_probability {
            glitches.clear();
            continue;

        }

        // start index into sample data for point of glitch (based on whole note, half note, quarter note etc)
        let mutateidx = (everybeat as f64 * beatidx as f64) as usize;

        // previous glitches are erased for next potential glitch event.
        glitches.clear();

        // How long the glitch event will last (half note, quarter note, etc)
        let glitch_duration_samples: usize = glitch_durations_in_samples[generator.gen::<usize>() % glitch_durations_in_samples.len()] as usize;

        // create glitch artifact: populates a vector with glitch samples and repetitions.
        glitch_iterative(&mut generator, &mut glitches, recurse_probability, glitch_duration_samples, 1);

        let mut mutation_index = 0;

        let mut cutoff = 0;

        let mut kill = false;

        let test_samples = 512;

        if glitch_probability == 1.0 {

            for test_sample in 0..test_samples {

                if mutateidx + test_sample >= mutated.len() {

                    break;

                }

                mutated[mutateidx + test_sample] = generator.gen::<i32>() % 32767;

            }

        } else {

            for nglitched_samples in &glitches {

                for glxsidx in 0..*nglitched_samples {

                    if mutation_index + mutateidx < mutated.len() {

                        mutated[mutateidx + mutation_index] = data[mutateidx + glxsidx];

                        mutation_index += 1;

                        cutoff += 1;

                        if cutoff > glitch_duration_samples {

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

    }

    for m_sample in mutated {

        wavwriter.write_sample(m_sample).expect("Failed to write sample");

    }

    wavwriter.finalize().unwrap();

}



