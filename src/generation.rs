use rand::{Rng, seq::SliceRandom, thread_rng};
use serde::{Serialize, Deserialize};

use crate::{
    categorize, 
    node::Node, 
    activation::ActivationFunction, 
    DEBUG, 
    error::DarjeelingError,
    input::Input, 
    types::Types::{Boolean, Integer, Float}, 
};

/// The top-level neural network struct
/// Sensor and answer represents which layer sensor and answer are on
#[derive(Debug, Serialize, Deserialize)]
pub struct NeuralNetwork {
    node_array: Vec<Vec<Node>>,
    sensor: Option<usize>,
    answer: Option<usize>,
    parameters: Option<u128>,
    activation_function: ActivationFunction
}

impl NeuralNetwork {
    
    /// Constructor function for the neural network
    /// Fills a Neural Network's node_array with empty nodes. 
    /// Initializes random starting link and bias weights between -.5 and .5
    /// 
    /// ## Params
    /// - Inputs: The number of sensors in the input layer
    /// - Hidden: The number of hidden nodes in each layer
    /// - Answer: The number of answer nodes, or possible categories
    /// - Hidden Layers: The number of different hidden layers
    /// 
    /// ## Examples
    /// ``` rust
    /// use darjeeling::{
    ///     activation::ActivationFunction,
    ///     categorize::NeuralNetwork
    /// };
    /// 
    /// let inputs: i32 = 10;
    /// let hidden: i32 = 40;
    /// let answer: i32 = 2;
    /// let hidden_layers: i32 = 1;
    /// let mut net: NeuralNetwork = NeuralNetwork::new(inputs, hidden, answer, hidden_layers, ActivationFunction::Sigmoid);
    /// ```
    pub fn new(pixel_input: i32, hidden_num: i32, pixel_output: i32, hidden_layers: i32, activation_function: ActivationFunction) -> NeuralNetwork {
        let mut net: NeuralNetwork = NeuralNetwork { node_array: vec![], sensor: Some(0), answer: Some(hidden_layers as usize + 1), parameters: None, activation_function};
        let mut rng = rand::thread_rng();
        net.node_array.push(vec![]);    
        for _i in 0..pixel_input {
            net.node_array[net.sensor.unwrap()].push(Node::new(&vec![], None));
        }

        for i in 1..hidden_layers + 1 {
            let mut hidden_vec:Vec<Node> = vec![];
            let hidden_links = net.node_array[(i - 1) as usize].len();
            if DEBUG { println!("Hidden Links: {:?}", hidden_links) }
            for _j in 0..hidden_num{
                hidden_vec.push(Node { link_weights: vec![], link_vals: vec![], links: hidden_links, err_sig: None, correct_answer: None, cached_output: None, category: None, b_weight: None });
            }
            net.node_array.push(hidden_vec);
        }

        net.node_array.push(vec![]);
        let answer_links = net.node_array[hidden_layers as usize].len();
        println!("Answer Links: {:?}", answer_links);
        for _i in 0..pixel_output {
            net.node_array[net.answer.unwrap()].push(Node { link_weights: vec![], link_vals: vec![], links: answer_links, err_sig: None, correct_answer: None, cached_output: Some(0.0), category: None, b_weight: None });
        }
        
        for layer in &mut net.node_array{
            for node in layer{
                node.b_weight = Some(rng.gen_range(-0.5..0.5));
                if DEBUG { println!("Made it to pushing link weights") }
                for _i in 0..node.links {
                    node.link_weights.push(rng.gen_range(-0.5..0.5));
                    node.link_vals.push(None);
                }
            }
        }
        let mut params = 0;
        for i in 0..net.node_array.len() {
            for j in 0..net.node_array[i].len() {
                params += 1 + net.node_array[i][j].links as u128;
            }
        }
        net.parameters = Some(params);
        net
    }
    // So the idea here is we have two models.
    // One that tells generated and ungenerated models apart, and one that generates images
    // The distinguishing model has to revceive the generative images, then 'prove its worth' every epoch by training so that it can distinguish images.
    // Then the generative model adjusts accordingly
    // Then on the next epoch, the generative model generates more
    pub fn learn(&mut self, data: &mut Vec<Input>, learning_rate: f32, name: &str) -> Result<String, DarjeelingError> {
        let mut epochs: f32 = 0.0;
        let hidden_layers = self.node_array.len() - 2;

        let mut distinguishing_model = &categorize::NeuralNetwork::new(8, 64, 2, 1, ActivationFunction::Sigmoid);
        let mut model_name: Option<String> = None;
        let mut outputs: Vec<Input> = vec![];
        for _i in 0..100 {

            data.shuffle(&mut thread_rng());

            for line in 0..data.len() {
                if DEBUG { println!("Training Checkpoint One Passed") }

                self.push_downstream(data, line as i32);
                
                let mut output = vec![];
                for i in 0..self.node_array[self.answer.unwrap()].len() {
                    output.push(self.node_array[self.answer.unwrap()][i].output(&self.activation_function));
                }
                outputs.push(Input::new(output, Boolean(false)));
            }
            if model_name.is_some() {
                let mut new_model = categorize::NeuralNetwork::read_model(model_name.unwrap()).unwrap();
                model_name = match new_model.learn(
                    &mut outputs, 
                    vec![Boolean(true), Boolean(false)], 
                    learning_rate, name) 
                    {
                        Ok(name) => Some(name.1),
                        Err(error) => return Err(DarjeelingError::DisinguishingModel(error))
                    };
                distinguishing_model = &new_model;
            } else {
                let mut new_model = categorize::NeuralNetwork::read_model(model_name.unwrap()).unwrap();
                model_name = match new_model.learn(data, vec![Boolean(true), Boolean(false)], learning_rate, name) {
                    Ok(name) => Some(name.1),
                    Err(error) => return Err(DarjeelingError::DisinguishingModel(error))
                };
                distinguishing_model = &new_model;
            }

            // model_name: (String, f32) = match distinguishing_model.learn(
            //         &mut outputs, 
            //         vec![Types::Boolean(true), Types::Boolean(false)], 
            //         learning_rate, name) 
            // {
            //     Ok(name) => name,
            //     Err(error) => return Err(error)
            // };
            
            self.backpropogate(learning_rate, hidden_layers as i32);

            // let _old_err_percent = err_percent;
            epochs += 1.0;
            println!("Epoch: {:?}", epochs);
            //if err_percent - old_err_percent < 0.00000001 { break; }

        }
        // #[allow(unused_mut)]
        // let mut model_name: String;
        // match self.write_model(&name) {

        //     Ok(m_name) => {
        //         model_name = m_name;
        //     },

        //     Err(error) => return Err(error)
        // }

        // println!("Training: Finished with accuracy of {:?}/{:?} or {:?} percent after {:?} epochs", sum, count, err_percent, epochs);

        Ok(String::new())
    }

    /// Passes in data to the sensors, pushs data 'downstream' through the network
    fn push_downstream(&mut self, data: &mut Vec<Input>, line: i32) {

        // Passes in data for input layer
        for i in 0..self.node_array[self.sensor.unwrap()].len() {
            let input  = data[line as usize].inputs[i];

            self.node_array[self.sensor.unwrap()][i].cached_output = Some(input);
        }

        // Feed-forward values for hidden and output layers
        for layer in 1..self.node_array.len() {

            for node in 0..self.node_array[layer].len() {

                for prev_node in 0..self.node_array[layer-1].len() {
                    
                    // self.node_array[layer][node].link_vals.push(self.node_array[layer-1][prev_node].cached_output.unwrap());
                    self.node_array[layer][node].link_vals[prev_node] = Some(self.node_array[layer-1][prev_node].cached_output.unwrap());
                    // I think this line needs to be un-commented
                    self.node_array[layer][node].output(&self.activation_function);
                    if DEBUG { if layer == self.answer.unwrap() { println!("Ran output on answer {:?}", self.node_array[layer][node].cached_output) } }
                }
                self.node_array[layer][node].output(&self.activation_function);
            }
        }
    }

    /// Finds the index and the brightest node in an array and returns it
    fn largest_node(&self) -> usize {
        let mut largest_node = 0;
        for node in 0..self.node_array[self.answer.unwrap()].len() {
            if self.node_array[self.answer.unwrap()][node].cached_output > self.node_array[self.answer.unwrap()][largest_node].cached_output {
                largest_node = node;
            }
        }

        largest_node
    }
    /// Goes back through the network adjusting the weights of the all the neurons based on their error signal
    fn backpropogate(&mut self, learning_rate: f32, hidden_layers: i32) {

        for answer in 0..self.node_array[self.answer.unwrap()].len() {
            if DEBUG { println!("Node: {:?}", self.node_array[self.answer.unwrap()][answer]); }

            self.node_array[self.answer.unwrap()][answer].compute_answer_err_sig();

            if DEBUG { println!("Error: {:?}", self.node_array[self.answer.unwrap()][answer].err_sig.unwrap()) }
        }

        self.adjust_hidden_weights(learning_rate, hidden_layers);

        // Adjusts weights for answer neurons
        for answer in 0..self.node_array[self.answer.unwrap()].len() {
            self.node_array[self.answer.unwrap()][answer].adjust_weights(learning_rate);
        }
    }

    #[allow(non_snake_case)]
    /// Adjusts the weights of all the hidden neurons in a network
    fn adjust_hidden_weights(&mut self, learning_rate: f32, hidden_layers: i32) {

        // HIDDEN represents the layer, while hidden represents the node of the layer
        for HIDDEN in 1..(hidden_layers + 1) as usize {            
            for hidden in 0..self.node_array[HIDDEN].len() {
                self.node_array[HIDDEN][hidden].err_sig = Some(0.0);
                for next_layer in 0..self.node_array[HIDDEN + 1 ].len() {
                    let next_weight = self.node_array[HIDDEN + 1][next_layer].link_weights[hidden];
                    self.node_array[HIDDEN + 1][next_layer].err_sig = match self.node_array[HIDDEN + 1][next_layer].err_sig.is_none() {
                        true => {
                            Some(0.0)
                        }, 
                        false => {
                            self.node_array[HIDDEN + 1][next_layer].err_sig
                        }
                    };
                    // This changes based on the activation function
                    self.node_array[HIDDEN][hidden].err_sig = Some(self.node_array[HIDDEN][hidden].err_sig.unwrap() + (self.node_array[HIDDEN + 1][next_layer].err_sig.unwrap() * next_weight));
                    if DEBUG { 
                        println!("next err sig {:?}", self.node_array[HIDDEN + 1][next_layer].err_sig.unwrap());
                        println!("next weight {:?}", next_weight);
                    }
                }
                let hidden_result = self.node_array[HIDDEN][hidden].cached_output.unwrap();
                let multiplied_value = self.node_array[HIDDEN][hidden].err_sig.unwrap() * (hidden_result) * (1.0 - hidden_result);
                if DEBUG { println!("new hidden errsig multiply: {:?}", multiplied_value); }
                self.node_array[HIDDEN][hidden].err_sig = Some(multiplied_value);

                if DEBUG { 
                    println!("\nLayer: {:?}", HIDDEN);
                    println!("Node: {:?}", hidden) 
                }

                self.node_array[HIDDEN][hidden].adjust_weights(learning_rate);
            }
        }
    }
}