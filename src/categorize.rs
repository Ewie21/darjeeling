use crate::{
    DEBUG,
    error::DarjeelingError,
    types::Types,
    node::Node,
    input::Input,
    activation::ActivationFunction
};
use std::{fs, path::Path};
use serde::{Deserialize, Serialize};
use rand::{Rng, seq::SliceRandom, thread_rng};
use rayon::prelude::*;

macro_rules! dbg_println {
    // `()` indicates that the macro takes no argument.
    ($($arg:tt)*) => {
        if DEBUG { println!($($arg)*) }
    };
}

/// The top-level neural network struct
/// sensor and answer represents which layer sensor and answer are on
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeuralNetwork {
    node_array: Vec<Vec<Node>>,
    sensor: Option<usize>,
    answer: Option<usize>,
    parameters: Option<u128>,
    activation_function: ActivationFunction
}
#[warn(clippy::unwrap_in_result)]
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
    /// activation::ActivationFunction,
    /// categorize::NeuralNetwork
    /// };
    /// 
    /// let inputs: i32 = 10;
    /// let hidden: i32 = 40;
    /// let answer: i32 = 2;
    /// let hidden_layers: i32 = 1;
    /// let mut net: NeuralNetwork = NeuralNetwork::new(inputs, hidden, answer, hidden_layers, ActivationFunction::Sigmoid);
    /// ```
    pub fn new(input_num: i32, hidden_num: i32, answer_num: i32, hidden_layers: i32, activation_function: ActivationFunction) -> NeuralNetwork {
        let mut net: NeuralNetwork = NeuralNetwork { node_array: vec![], sensor: Some(0), answer: Some(hidden_layers as usize + 1), parameters: None, activation_function};
        let mut rng = rand::thread_rng();
        net.node_array.push(vec![]);    
        for _i in 0..input_num {
            net.node_array[net.sensor.unwrap()].push(Node::new(&vec![], None));
        }

        for i in 1..hidden_layers + 1 {
            let mut hidden_vec:Vec<Node> = vec![];
            let hidden_links = net.node_array[(i - 1) as usize].len();
            dbg_println!("Hidden Links: {:?}", hidden_links);
            for _j in 0..hidden_num{
                hidden_vec.push(Node { link_weights: vec![], link_vals: vec![], links: hidden_links, err_sig: None, correct_answer: None, cached_output: None, category: None, b_weight: None });
            }
            net.node_array.push(hidden_vec);
        }

        net.node_array.push(vec![]);
        let answer_links = net.node_array[hidden_layers as usize].len();
        println!("Answer Links: {:?}", answer_links);
        for _i in 0..answer_num {
            net.node_array[net.answer.unwrap()].push(Node { link_weights: vec![], link_vals: vec![], links: answer_links, err_sig: None, correct_answer: None, cached_output: Some(0.0), category: None, b_weight: None });
        }
        
        net.node_array
            .iter_mut()
            .for_each(|layer| {
                layer
                    .iter_mut()
                    .for_each(|mut node| {
                        node.b_weight = Some(rng.gen_range(-0.5..0.5));
                        dbg_println!("Made it to pushing link weights");
                        (0..node.links)
                            .into_iter()
                            .for_each(|_| {
                                node.link_weights.push(rng.gen_range(-0.5..0.5));
                                node.link_vals.push(None);
                            })
                    })
            });
        let mut params = 0;
        (0..net.node_array.len())
            .into_iter()
            .for_each(|i| {
                (0..net.node_array[i].len())
                    .into_iter()
                    .for_each(|j| {
                        params += 1 + net.node_array[i][j].links as u128;
                    })
            });
        net.parameters = Some(params);
        net
    }

    /// Trains the neural network model to be able to categorize data in a dataset
    /// 
    /// ## Params
    /// - Data: List of inputs
    /// - Categories: List of Strings, each denoting an answer category. 
    /// The number of answer nodes should be the same of the number of categories
    /// - Learning Rate: The modifier that is applied to link weights as they're adjusted.
    /// Try fiddling with this one, but -1.5 - 1.5 is recommended to start.
    /// 
    /// ## Returns
    /// The falable name of the model that this neural network trained
    /// 
    /// ## Err
    /// - ### WriteModelFailed
    /// There was a problem when saving the model to a file
    /// - ### ModelNameAlreadyExists
    /// The random model name chosen already exists
    /// 
    /// Change the name or retrain
    /// - ### UnknownError
    /// Not sure what happened, but something failed
    /// 
    /// Make an issue on the [darjeeling](https://github.com/Ewie21/darjeeling) github page
    /// 
    /// Or contact me at elocolburn@comcast.net
    /// 
    /// ## Examples
    /// ```ignore
    /// use darjeeling::{
    /// categorize::NeuralNetwork,
    /// activation::ActivationFunction,
    /// input::Input, 
    /// // This file may not be avaliable
    /// // Everything found here will be hyper-specific to your project.
    /// tests::{categories_str_format, xor_file}
    /// };
    /// 
    /// // A file containing all possible inputs and correct outputs still needs to be make by you
    /// // 0 0;0
    /// // 0 1;1
    /// // 1 0;1
    /// // 1 1;0
    /// // You also need to write the file input function
    /// // Automatic file reading and formatting function coming soon
    /// let categories: Vec<String> = categories_str_format(vec!["0", "1"]);
    /// let mut data: Vec<Input> = xor_file();
    /// let mut net = NeuralNetwork::new(2, 2, 2, 1, ActivationFunction::Sigmoid);
    /// let learning_rate = 1.0;
    /// let (model_name, error_percentage, mse) = net.learn(&mut data, categories, learning_rate, "xor", 99.0).unwrap();
    /// ```
    pub fn learn<'b>(
        &'b mut self, 
        data: &mut Vec<Input>, 
        categories: Vec<Types>, 
        learning_rate: f32, 
        name: &str,
        target_err_percent: f32
    ) -> Result<(String, f32, f32), DarjeelingError> {
        let mut epochs: f32 = 0.0;
        let mut sum: f32 = 0.0;
        let mut count: f32 = 0.0;
        let mut err_percent: f32 = 0.0;
        let hidden_layers = self.node_array.len() - 2;
        let mut mse: f32 = 0.0;

        self.categorize(categories);
        
        while err_percent < target_err_percent {
            count = 0.0;
            sum = 0.0;
            data.shuffle(&mut thread_rng());

            for line in 0..data.len() {
                dbg_println!("Training Checkpoint One Passed");
                
                self.assign_answers(&mut data[line]);

                self.push_downstream(data, line);

                dbg_println!("Sum: {:?} Count: {:?}", sum, count);

                self.self_analysis(&mut Some(epochs), &mut sum, &mut count, data, &mut mse, line);

                dbg_println!("Sum: {:?} Count: {:?}", sum, count);
                
                self.backpropogate(learning_rate, hidden_layers as i32);
            }

            // let _old_err_percent = err_percent;
            err_percent = (sum/count) * 100.0;
            epochs += 1.0;
            println!("Epoch: {:?}", epochs);
            println!("Training Accuracy: {:?}", err_percent);
            //if err_percent - old_err_percent < 0.00000001 { break; }

        }
        let model_name: String;
        match self.write_model(&name) {
            Ok(m_name) => {
                model_name = m_name;
            },
            Err(err) => return Err(err),
        }

        println!("Training: Finished with accuracy of {:?}/{:?} or {:?} percent after {:?} epochs\nmse: {}", sum, count, err_percent, epochs, mse);

        Ok((model_name, err_percent, mse))
    }

    /// Tests a pretrained model
    pub fn test(mut data: Vec<Input>, categories: Vec<Types>, model_name: String) -> Result<Vec<Types>, DarjeelingError> {
        let mut sum = 0.0;
        let mut count = 0.0;
        // let mut category: Option<Types> = None;
        let mut answers: Vec<Types> = vec![];
        let mut mse = 0.0;

        let mut net: NeuralNetwork = match NeuralNetwork::read_model(model_name.clone()) {

            Ok(net) => net,
            Err(error) => return Err(DarjeelingError::ReadModelFunctionFailed(model_name, Box::new(error)))
        };

        for node in 0..net.node_array[net.answer.unwrap()].len() {
            net.node_array[net.answer.unwrap()][node].category = Some(categories[node].clone());
            dbg_println!("{:?}", net.node_array[net.answer.unwrap()][node].category);
        }

        for line in 0..data.len() {
            dbg_println!("Testing Checkpoint One Passed");
            if data[line].answer.is_some() {
                net.assign_answers(&mut data[line]);
            }
            ; // Do we actually want to do this?
            net.push_downstream(&mut data, line);
            dbg_println!("Sum: {:?} Count: {:?}", sum, count);            
            answers.push((
                Some(
                    net.self_analysis(&mut None, &mut sum, &mut count, &mut data, &mut mse, line).0
                ))
                .clone().expect("Wrapped in Some()"));

            dbg_println!("Sum: {:?} Count: {:?}", sum, count);

            // println!("Correct answer: {:?}", data[line].answer)
        }

        // let _old_err_percent = err_percent;
        let err_percent: f32 = (sum/count) * 100.0;
        mse /= count;
        println!("Testing: Finished with accuracy of {:?}/{:?} or {:?} percent\nMSE: {}", sum, count, err_percent, mse);

        Ok(answers)
    }

    /// Assigns categories to answer nodes based on a list of given categories
    fn categorize(&mut self, categories: Vec<Types>) {
        let mut count: usize = 0;
        self.node_array[self.answer.unwrap()]
            .iter_mut()
            .for_each(|node| {
                node.category = Some(categories[count].clone());
                count += 1;
            });
    }
    
    fn assign_answers(&mut self, input: &mut Input) {
        let _ = self.node_array[self.answer.unwrap()]
            .par_iter_mut()
            .for_each(|mut node| {
                println!("{:?}", input);
                if node.category.as_ref().unwrap() == input.answer.as_ref().unwrap() {
                    node.correct_answer = Some(1.0);
                } else {
                    node.correct_answer = Some(0.0);
                }
            });
    }

    /// Passes in data to the sensors, pushs data 'downstream' through the network
    fn push_downstream(&mut self, data: &mut Vec<Input>, line: usize) {
        // Passes in data for input layer
        (0..self.node_array[self.sensor.unwrap()].len())
            .into_iter()
            .for_each(|i| {
                let input  = data[line].inputs[i];
                self.node_array[self.sensor.unwrap()][i].cached_output = Some(input);
            });

        // Feed-forward values for hidden and output layers
        (1..self.node_array.len())
            .into_iter()
            .for_each(|layer_i| {
                (0..self.node_array[layer_i].len())
                    .into_iter()
                    .for_each(|node_i| {
                        (0..self.node_array[layer_i - 1].len())
                            .into_iter()
                            .for_each(|prev_node_i| {
                                // self.node_array[layer][node].link_vals.push(self.node_array[layer-1][prev_node].cached_output.unwrap());
                                self.node_array[layer_i][node_i].link_vals[prev_node_i] = Some(self.node_array[layer_i-1][prev_node_i].cached_output.unwrap());
                                // I think this line needs to be un-commented
                                self.node_array[layer_i][node_i].output(&self.activation_function);
                                if layer_i == self.answer.unwrap() { dbg_println!("Ran output on answer {:?}", self.node_array[layer_i][node_i].cached_output); }
                            });
                        self.node_array[layer_i][node_i].output(&self.activation_function);
                    });
            });
    }

    /// Analyses the chosen answer node's result.
    /// Also increments sum and count
    fn self_analysis<'b>(
        &'b self, 
        epochs: &mut Option<f32>, 
        sum: &'b mut f32, 
        count: &'b mut f32, 
        data: &mut Vec<Input>, 
        mse: &mut f32, 
        line: usize
    ) -> (Types, f32) {
        // println!("answer {}", self.answer.unwrap());
        // println!("largest index {}", self.largest_node());
        // println!("{:?}", self);
        let brightest_node: &Node = &self.node_array[self.answer.unwrap()][self.largest_node()];
        let brightness: f32 = brightest_node.cached_output.unwrap();

        if !(epochs.is_none()) { // This won't happen during testing
            if epochs.unwrap() % 10.0 == 0.0 && epochs.unwrap() != 0.0 {
                println!("\n-------------------------\n");
                println!("Epoch: {:?}", epochs);
                println!("Category: {:?} \nBrightness: {:?}", brightest_node.category.as_ref().unwrap(), brightness);
                if DEBUG {
                    let dimest_node: &Node = &self.node_array[self.answer.unwrap()][self.node_array[self.answer.unwrap()].len()-1-self.largest_node()];
                    println!("Chosen category: {:?} \nDimest Brightness: {:?}", dimest_node.category.as_ref().unwrap(), dimest_node.cached_output.unwrap());
                }
            }
        }

        dbg_println!("Category: {:?} \nBrightness: {:?}", brightest_node.category.as_ref().unwrap(), brightness);
        if data[line].answer.is_some() {
            if brightest_node.category.as_ref().unwrap().eq(&data[line].answer.as_ref().unwrap()) {
                dbg_println!("Sum++");
                *sum += 1.0;
            }
            *count += 1.0;
        }

        (brightest_node.category.clone().unwrap(), NeuralNetwork::calculate_err_for_generation_model(mse, *sum, *count))
    }

    fn calculate_err_for_generation_model(mse: &mut f32, sum: f32, count: f32) -> f32 {
        *mse += f32::powi(sum - count, 2);
        *mse
    }

    /// Finds the index and the brightest node in an array and returns it
    fn largest_node(&self) -> usize {
        let mut largest_node = 0;
        (0..self.node_array[self.answer.unwrap()].len())
            .into_iter()
            .for_each(|node_i| {
                if self.node_array[self.answer.unwrap()][node_i].cached_output > self.node_array[self.answer.unwrap()][largest_node].cached_output {
                    largest_node = node_i;
                }
            });
        largest_node
    }
    /// Goes back through the network adjusting the weights of the all the neurons based on their error signal
    fn backpropogate(&mut self, learning_rate: f32, hidden_layers: i32) {
        for answer in 0..self.node_array[self.answer.unwrap()].len() {
            dbg_println!("Node: {:?}", self.node_array[self.answer.unwrap()][answer]);
            self.node_array[self.answer.unwrap()][answer].compute_answer_err_sig(&self.activation_function);
            dbg_println!("Error: {:?}", self.node_array[self.answer.unwrap()][answer].err_sig.unwrap());
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

                    dbg_println!("next err sig {:?}", self.node_array[HIDDEN + 1][next_layer].err_sig.unwrap());
                    dbg_println!("next weight {:?}", next_weight);
                }
                let hidden_result = self.node_array[HIDDEN][hidden].cached_output.unwrap();
                let multiplied_value = self.node_array[HIDDEN][hidden].err_sig.unwrap() * (hidden_result) * (1.0 - hidden_result);
                dbg_println!("new hidden errsig multiply: {:?}", multiplied_value);
                self.node_array[HIDDEN][hidden].err_sig = Some(multiplied_value);

                dbg_println!("\nLayer: {:?}", HIDDEN);
                dbg_println!("Node: {:?}", hidden);

                self.node_array[HIDDEN][hidden].adjust_weights(learning_rate);
            }
        }
    }

    /// Serializes a trained model so it can be used later
    /// 
    /// ## Returns
    /// The name of the model
    /// 
    /// ## Err
    /// ### WriteModelFailed: 
    /// Wraps the models name
    /// ### UnknownError: 
    /// Wraps error
    ///  
    pub fn write_model(&mut self, name: &str) -> Result<String, DarjeelingError> {
        
        let mut rng = rand::thread_rng();
        let file_num: u32 = rng.gen();
        let model_name: String = format!("model_{}_{}.darj", name, file_num);

        match Path::new(&model_name).try_exists() {
            Ok(false) => {
                let _file: fs::File = fs::File::create(&model_name).unwrap();
                let mut serialized = "".to_string();
                println!("write, length: {}", self.node_array.len());
                for i in 0..self.node_array.len() {
                    if i != 0 {
                        let _ = serialized.push_str("lb\n");
                    }
                    for j in 0..self.node_array[i].len() {
                        for k in 0..self.node_array[i][j].link_weights.len() {
                            print!("{}", self.node_array[i][j].link_weights[k]);
                            if k == self.node_array[i][j].link_weights.len() - 1 {
                                let _ = serialized.push_str(format!("{}", self.node_array[i][j].link_weights[k]).as_str());
                            } else {
                                let _ = serialized.push_str(format!("{},", self.node_array[i][j].link_weights[k]).as_str());
                            }                        
                        }
                        let _ = serialized.push_str(format!(";{}", self.node_array[i][j].b_weight.unwrap().to_string()).as_str()); 
                        let _ = serialized.push_str("\n");
                    }
                }
                serialized.push_str("lb\n");                    
                serialized.push_str(format!("{}", self.activation_function).as_str());
                println!("Serialized: {:?}", serialized);
                println!("{}", model_name);
                match fs::write(&model_name, serialized) {
                    Ok(()) => {
                        println!("Model {:?} Saved", file_num);
                        Ok(model_name)
                    },
                    Err(_error) => {
                        Err(DarjeelingError::WriteModelFailed(model_name))
                    }
                }
            },
            Ok(true) => {
                self.write_model(name)
            },
            Err(error) => Err(DarjeelingError::UnknownError(error.to_string()))
        }
    }

    /// Reads a serizalized Neural Network
    /// 
    /// ## Params
    /// - Model Name: The name(or more helpfully the path) of the model to be read
    /// 
    /// ## Returns
    /// A neural network read from a serialized .darj file
    /// 
    /// ## Err
    /// If the file cannnot be read, or if the file does not contain a valid serialized Neural Network
    pub fn read_model(model_name: String) -> Result<NeuralNetwork, DarjeelingError> {
        println!("Loading model");
        // Err if the file reading fails
        let serialized_net: String = match fs::read_to_string(&model_name) {
            
            Ok(serizalized_net) => serizalized_net,
            Err(error) => return Err(DarjeelingError::ReadModelFailed(model_name.clone() + ";" +  &error.to_string()))
        };
        let mut node_array: Vec<Vec<Node>> = vec![];
        let mut layer: Vec<Node> = vec![];
        let mut activation: Option<ActivationFunction> = None;
        for i in serialized_net.lines() {
            match i {
                "sigmoid" => activation = Some(ActivationFunction::Sigmoid),

                "linear" => activation = Some(ActivationFunction::Linear),

                // "tanh" => activation = Some(ActivationFunction::Tanh),

                // "step" => activation = Some(ActivationFunction::Step),

                _ => {
                
                    if i.trim() == "lb" {
                        node_array.push(layer.clone());
                        // println!("pushed layer {:?}", layer.clone());
                        layer = vec![];
                        continue;
                    }
                    #[allow(unused_mut)]
                    let mut node: Option<Node>;
                    if node_array.len() == 0 {
                        let b_weight: Vec<&str> = i.split(";").collect();
                        // println!("b_weight: {:?}", b_weight);
                        node = Some(Node::new(&vec![], match b_weight[1].parse()
                        {
                            Ok(weight) => Some(weight),
                            Err(err) => return Err(DarjeelingError::InvalidNodeValueRead(err.to_string() + "; Bias: " + b_weight[1])),
                        }));
                    } else {
                        let node_data: Vec<&str> = i.trim().split(";").collect();
                        let str_weight_array: Vec<&str> = node_data[0].split(",").collect();
                        let mut weight_array: Vec<f32> = vec![];
                        let b_weight: &str = node_data[1];
                        // println!("node_data: {:?}", node_data);
                        // println!("array {:?}", str_weight_array);
                        for weight in 0..str_weight_array.len() {
                            // println!("testing here {:?}", str_weight_array[weight]);
                            let val: f32 = match str_weight_array[weight].parse() 
                            {
                                Ok(v) => v,
                                Err(err) => return Err(DarjeelingError::InvalidNodeValueRead(err.to_string() + "; Weight: " + str_weight_array[weight])),
                            };
                            weight_array.push(val);
                        }
                        // print!("{}", b_weight);
                        node = Some(Node::new(&weight_array, match b_weight.parse()
                        {
                            Ok(weight) => Some(weight),
                            Err(err) => return Err(DarjeelingError::InvalidNodeValueRead(err.to_string() + " ;" + b_weight)),
                        }));
                    }
                    
                    layer.push(node.expect("Both cases provide a Some value for node"));
                    // println!("layer: {:?}", layer.clone())
                }
            }  
        }
        //println!("node array size {}", node_array.len());
        let sensor: Option<usize> = Some(0);
        let answer: Option<usize> = Some(node_array.len() - 1);
        
        let net = NeuralNetwork {
            node_array,
            sensor,
            answer,
            parameters: None,
            activation_function: match activation 
            {
                Some(acti) => acti,
                None => return Err(DarjeelingError::ActivationFunctionNotRead(format!("While attempting to read file {}", model_name))),
            }
        };
        // println!("node array {:?}", net.node_array);

        Ok(net)
    }

    fn set_activation_func(&mut self, new_activation_function: ActivationFunction) {
        self.activation_function = new_activation_function;
    }
}
