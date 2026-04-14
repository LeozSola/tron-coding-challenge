use crate::engine::{player_id, prelude::*};


pub struct CnnmlBot{
    args: BotArgs,
    // model: ModelEngine,
}

impl Bot for CnnmlBot{
    fn new(args: BotArgs)->Self {
        CnnmlBot{args}
    }

    fn next_action(&mut self, game_state: &GameState) -> Direction {
        todo!()

    }
}

/*
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X
    X, X, X, X, X, X, X, X, X


    X, X, X, X, X, X, X
    X, X, X, X, X, X, X
    X, X, X, X, X, X, X
    X, X, X, X, X, X, X
    X, X, X, X, X, X, X
    X, X, X, X, X, X, X
    X, X, X, X, X, X, X


    X, X, X, X, X
    X, X, X, X, X
    X, X, X, X, X
    X, X, X, X, X
    X, X, X, X, X
*/




type ModelInput = [f32; 9*9];
struct ModelEngine<'a>{
    grid: Option<&'a Grid>,
    player: PlayerId,
    model: Model,
}
impl<'a> ModelEngine<'a> {
    fn new(args: BotArgs)->Self{
        Self{
            grid: None,
            player: args.my_player(),
            model: Default::default()
        }
    }
    fn get_model_input(&self)->ModelInput{
        todo!()

    }
    fn parse_model_output(output: ModelOutput)->Direction{
        todo!()

    }
    fn get_model_next_step(&self)->Direction{
        Self::parse_model_output(self.model.forward(Self::get_model_input(&self)))
    }
    fn set_engine_to_state(&mut self, s: &GameState){

    }
}


enum ModelOutput{
    Left, Right, Up
}
#[derive(Default)]
struct Model{
    first: ConvolutionLayer, // 9x9 -> 7x7
    second: ConvolutionLayer, // 7x7 -> 5x5
    thid: DenseLayer, // 5x5 -> 3 (left up right)
}
impl Model {
    fn forward(&self, x: ModelInput)->ModelOutput {
        todo!()
    }
}

struct ConvolutionLayer{
    weights: [f32; 9],
}
impl ConvolutionLayer {
    // invariant: slice has length equal to a perfect square
    fn forward(slice: [f32; 9])->Vec<f32>{
        todo!()
        
    }
}
impl Default for ConvolutionLayer{
    fn default() -> Self {
        Self { weights: [1.0; 9] }
    }
}
struct DenseLayer{
    weights: [f32; 25],
    bias: [f32; 25],
}
impl DenseLayer {
    fn forward(slice: [f32; 25])->[f32; 3]{
        todo!()
        
    }
}
impl Default for DenseLayer{
    fn default() -> Self {
        Self { weights: [1.0; 25], bias: [0.0; 25] }
    }
}