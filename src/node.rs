use crate::{activation::ActivationFunction, dbg_println, DEBUG};
use serde::{Deserialize, Serialize};

/// Represents a node in the network
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Node {
    pub link_weights: Box<[f32]>,
    pub link_vals: Box<[f32]>,
    pub b_weight: f32,
    #[serde(skip_serializing)]
    pub err_sig: Option<f32>,
    #[serde(skip_serializing)]
    pub correct_answer: Option<f32>,
    #[serde(skip_serializing)]
    pub category: Option<String>,
    #[serde(skip_serializing)]
    pub cached_output: Option<f32>,
}

impl Node {
    pub fn new(link_weights: Box<[f32]>, b_weight: f32) -> Node {
        let link_vals: Box<[f32]> = (0..link_weights.len())
            .map(|_| 0.00)
            .collect::<Box<[f32]>>();

        Node {
            link_weights,
            link_vals,
            b_weight,
            err_sig: None,
            cached_output: None,
            correct_answer: None,
            category: None,
        }
    }

    fn input(&mut self) -> f32 {
        (0..self.link_weights.len())
            .into_iter()
            .map(|i| {
                dbg_println!("Link Val: {:?}", self.link_vals[i]);
                self.link_vals[i] * self.link_weights[i]
            })
            .sum::<f32>()
            * self.b_weight
    }

    pub fn output(&mut self, activation: ActivationFunction) -> f32 {
        match activation {
            ActivationFunction::Sigmoid => Node::sigmoid(self.input()),
            ActivationFunction::Linear => Node::linear(self.input()),
            ActivationFunction::Tanh => Node::tanh(self.input()),
            // ActivationFunction::Step => Node::step(self.input()),
        }
    }

    pub fn compute_answer_err_sig(&mut self, activation: ActivationFunction) -> f32 {
        let cached_output = self
            .cached_output
            .expect("Answer Node Missing Cached Output");

        let derivative: f32 = match activation {
            ActivationFunction::Sigmoid => cached_output * (1.0 - cached_output),
            ActivationFunction::Linear => 2.0,
            // Unsupported
            ActivationFunction::Tanh => 1.0, //- unsafe { std::intrinsics::powf32(y, 2.0) };
        };

        self.err_sig = Some((self.correct_answer.unwrap() - cached_output) * cached_output * derivative);
        self.err_sig.unwrap()
    }

    pub fn compute_answer_err_sig_gen(&mut self, mse: f32, activation: ActivationFunction) -> f32 {
        // This is where the derivative of the activation function goes I think
        let output = self
            .cached_output
            .expect("Answer Node Missing Cached Output");
        let derivative: f32 = match activation {
            ActivationFunction::Sigmoid => output * (1.0 - output),
            ActivationFunction::Linear => 2.0,
            ActivationFunction::Tanh => 1.0, //- unsafe { std::intrinsics::powf32(y, 2.0) };
        };
        self.err_sig = Some(mse * derivative);
        dbg_println!("Err Signal Post: {:?}", self.err_sig.unwrap());
        self.err_sig.unwrap()
    }

    pub fn adjust_weights(&mut self, learning_rate: f32) {
        let err_sig = self.err_sig.expect("Node has no error signal");
        self.b_weight += err_sig * learning_rate;
        self.link_weights = self.link_weights
            .iter()
            .enumerate()
            .map(|(link, link_weight)|
                link_weight + err_sig * self.link_vals[link] * learning_rate)
            .collect();
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + ((-x).exp()))
    }

    fn linear(x: f32) -> f32 {
        2.0 * x
    }

    fn tanh(x: f32) -> f32 {
        let e = std::f64::consts::E as f32;

        2.00 / (1.00 + e.powf(-2.00 * x)) - 1.00
    }

    fn step(x: f32) -> f32 {
        if x < 0.00 {
            -1.00
        } else {
            1.00
        }
    }
}
