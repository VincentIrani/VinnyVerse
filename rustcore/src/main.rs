// WELCOME TO THE VINNYVERSE ///////////////////////////////////////////////////////////////////////////////////////////////////
// This is the Rust core of the VinnyVerse, a virtual world where players and AI agents try their best to eat each other. //////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// File Imports ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
mod utils;
mod cell_def;

use cell_def::{Cell, CellKind};

// External Imports ////////////////////////////////////////////////////////////////////////////////////////////////////////////
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use futures_util::{StreamExt, SinkExt};

use std::io::{self, BufRead, Write, Read};
use std::fs::File;
use std::net::SocketAddr;

use serde::{Serialize, Deserialize};
use serde_json;

// Constant Definition /////////////////////////////////////////////////////////////////////////////////////////////////////////

// Variables/enums Definition //////////////////////////////////////////////////////////////////////////////////////////////////
//Server states
#[derive(Debug)]
enum ServerState {
    Idle,
    GeneratingWorld(usize), // World Generation in progress with size parameter
    SavingWorld(String), // World Saving in progress with filename
    LoadingWorld(String), // World Loading in progress with filename
    WorldRunning,
}

// Types of Commands
#[derive(Debug)]
enum Command {
    LoadWorld(String),
    SaveWorld(String),
    GenerateWorld(usize), // World Generated with size parameter
    StartWorldLoop,
    StopWorldLoop,
    Quit,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
enum UserInput{
    GenerateSoul {soul_id: String},
    MountSoul {soul_id: String},
    NameSoul {soul_id: String, name: String },
    DismountSoul {soul_id: String}, 
    Activate {soul_id: String, X: u32, Y: u32, power: i16},
    Build {soul_id: String, block_type: String, X: u32, Y: u32, dir: String, power: i16},
    UpdateBrain {soul_id: String, code: String},
    ReadBrain {soul_id: String},
}

#[derive(Serialize, Deserialize)]
pub struct WorldData {
    pub world: Vec<Vec<u8>>, // Placeholder for world data
    pub critter_layer: Vec<Vec<Cell>>, // Placeholder for critter layer
    pub soul_locations: Vec<(String, u32, u32)>, // Placeholder for soul locations
}

impl WorldData {
    pub fn save(&self, filename: &str) -> std::io::Result<()> {
        let encoded = bincode::serialize(self).unwrap();
        let mut file = File::create(filename)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn load(filename: &str) -> std::io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let state: WorldData = bincode::deserialize(&buffer).unwrap();
        Ok(state)
    }
}

// State Machine Transition Handler //////////////////////////////////////////////////////////////////////////////////////////////////
impl ServerState {
    fn handle_command(self, command: Command) -> ServerState {
        use ServerState::*;
        use Command::*;

        match (self, command) {
            (Idle, StartWorldLoop) => {
                println!("Starting world loop...");
                WorldRunning
            }
            (WorldRunning, StopWorldLoop) => {
                println!("Stopping world loop...");
                Idle
            }
            (Idle, GenerateWorld(size)) => {
                println!("Generating world with size: {}", size);
                GeneratingWorld(size)
            }
            (Idle, SaveWorld(filename)) => {
                println!("Saving world to file: {}", filename);
                SavingWorld(filename)
            }
            (Idle, LoadWorld(filename)) => {
                println!("Loading world from file: {}", filename);
                LoadingWorld(filename)
            }
            (state, cmd) => {
                println!("Command {:?} invalid in state {:?}", cmd, state);
                state
            }
        }
    }
}

// Main Loop ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
#[tokio::main]
async fn main() {

    // Create a channel for commands (could be from network or user input)
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();

    // Spawn a task to read user input commands from stdin
    let input_tx = cmd_tx.clone();
    tokio::spawn(async move {
        let mut input = String::new();

        loop {
            input.clear();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let cmd = match input.trim().to_lowercase().as_str() {
                "load_world" => {
                    let filename = read_file_name();
                    Command::LoadWorld(filename)
                },
                "save_world" => {
                    let filename = read_file_name();
                    Command::SaveWorld(filename)
                },
                "generate_world" => {
                    let size = read_world_size();
                    Command::GenerateWorld(size)
                },
                "start_world" => Command::StartWorldLoop,
                "stop_world" => Command::StopWorldLoop,
                "quit" => Command::Quit,
                _ => {
                    println!("Unknown command");
                    continue;
                }
            };

            if input_tx.send(cmd).is_err() {
                break;
            }
        }
    });

    // Initial state
    let mut state = ServerState::Idle;
    
    let (tx, mut rx) = mpsc::unbounded_channel::<UserInput>();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut ws_task_handle: Option<tokio::task::JoinHandle<()>> = None;
    
    let mut world_data = WorldData {
        world: vec![vec![0u8; 2]; 2], // Placeholder for world data
        critter_layer: vec![vec![Cell::empty(); 2]; 2], // Placeholder for critter layer
        soul_locations: Vec::new(), // Placeholder for soul locations
    };

    // This is the server loop
    loop {

        // Handle commands from the command channel
        while let Ok(cmd) = cmd_rx.try_recv() {
            if let Command::Quit = cmd {
                println!("Quitting server.");
                return;
            } else {
                println!("Received command: {:?}", cmd);
            }
            // Update state based on command
            state = state.handle_command(cmd);
        }

        match state {
            ServerState::Idle => {

            if let Some(handle) = ws_task_handle.take() {
                // Send shutdown signal
                let _ = shutdown_tx.send(true);

                // Await the task to finish
                handle.await.unwrap();
            }

            sleep(Duration::from_millis(7000)).await;

            }

            ServerState::GeneratingWorld(size) => {
                // Here you would add logic to generate the world
                println!("Generating world of size: {}", size);
               
                // Initialize the world and critter_layer with the specified size
                world_data.critter_layer = vec![vec![Cell::empty(); size]; size];
                world_data.world = utils::generate_world(size);
                // Transition to WorldRunning state after generating the world
                state = ServerState::Idle;
            }
            ServerState::SavingWorld(filename) => {
                // Here you would add logic to save the world
                println!("Saving world to file: {}", filename);
                if let Err(e) = world_data.save(&filename) {
                    println!("Failed to save world: {}", e);
                } else {
                    println!("World saved successfully.");
                }
                // Transition back to Idle state after saving
                state = ServerState::Idle;
            }
            ServerState::LoadingWorld(filename) => {
                // Here you would add logic to load the world
                println!("Loading world from file: {}", filename);
                match WorldData::load(&filename) {
                    Ok(loaded_world) => {
                        world_data = loaded_world;
                        println!("World loaded successfully.");
                    }
                    Err(e) => {
                        println!("Failed to load world: {}", e);
                    }
                }
                // Transition back to Idle state after loading
                state = ServerState::Idle;
            }
            ServerState::WorldRunning => {
                
                // Checking if the WebSocket listener task is running, and if not, spawn it
                if ws_task_handle.is_none() {
                    // Spawn the WebSocket listener task with necessary channels (tx, shutdown_rx)
                    let _ = shutdown_tx.send(false);
                    let shutdown_rx_clone = shutdown_rx.clone(); // clone receiver for the task
                    ws_task_handle = Some(spawn_ws_listener(tx.clone(), shutdown_rx_clone));
                }

                // Drain all messages currently buffered in rx
                let mut batch = Vec::new();
                while let Ok(msg) = rx.try_recv() {
                    batch.push(msg);
                }

                let mut build_que: Vec<UserInput> = Vec::new();

                println!("World loop got {} messages:", batch.len());
                for msg in batch {
                    match msg{
                        UserInput::GenerateSoul { soul_id } => {
                            println!("Generating soul with ID: {}", soul_id);
                            // Here you would add logic to generate a soul
                        }
                        UserInput::MountSoul { soul_id } => {
                            println!("Mounting soul with ID: {}", soul_id);
                            // Logic to mount a soul
                        }
                        UserInput::NameSoul { soul_id, name } => {
                            println!("Naming soul: {}", name);
                            // Logic to name a soul
                        }
                        UserInput::DismountSoul {soul_id}=> {
                            println!("Dismounting current soul");
                            // Logic to dismount a soul
                        }
                        UserInput::Activate {soul_id, X, Y, power } => {
                            println!("Activating at ({}, {}), power: {}", X, Y, power);
                            // Logic to activate something in the world
                        }
                        UserInput::Build {ref soul_id, ref block_type, X, Y, ref dir, power } => {
                            println!("Building {} at ({}, {}), direction: {}, power: {}", block_type, X, Y, dir, power);
                            build_que.push(msg);
                        }
                        UserInput::UpdateBrain {soul_id, code } => {
                            println!("Updating brain with code: {}", code);
                            // Logic to update brain code
                        }
                        UserInput::ReadBrain { soul_id } => {
                            println!("Reading brain state");
                            // Logic to read brain state
                        }
                    }
                }

                
                utils::the_sun(&mut world_data.world);

                println!("{:?}", build_que);

                utils::build_critters(&mut world_data.critter_layer, &mut build_que);

                println!("World size: {}x{}", world_data.world.len(), world_data.world[0].len());
        
                //utils::visualize_world_console(&world);
                utils::visualize_critter_layer(&world_data.critter_layer);

                sleep(Duration::from_millis(10000)).await;

            }
        }


    }
}

pub fn spawn_ws_listener(
    tx: mpsc::UnboundedSender<UserInput>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();
        println!("Server listening on 127.0.0.1:9001");

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            println!("New client: {}", addr);
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                let ws_stream = match accept_async(stream).await {
                                    Ok(ws) => ws,
                                    Err(e) => {
                                        println!("WebSocket handshake failed with {}: {}", addr, e);
                                        return;
                                    }
                                };

                                let mut read = ws_stream;

                                while let Some(msg_result) = read.next().await {
                                    match msg_result {
                                        Ok(msg) => {
                                            if msg.is_text() {
                                                let text = msg.to_text().unwrap();
                                                match serde_json::from_str::<UserInput>(text) {
                                                    Ok(user_input) => {
                                                        if tx.send(user_input).is_err() {
                                                            println!("Receiver dropped, closing client {}", addr);
                                                            break;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("Failed to parse JSON from client {}: {}", addr, e);
                                                    }
                                                }
                                            } else if msg.is_close() {
                                                println!("Client {} disconnected", addr);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            println!("Error receiving message from client {}: {:?}", addr, e);
                                            break;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            println!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        println!("Shutting down WebSocket listener...");
                        break;
                    }
                }
            }
        }
    })
}

fn read_world_size() -> usize {
    loop {
        print!("Enter world size: ");
        io::stdout().flush().unwrap(); // flush to show prompt immediately

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<usize>() {
            Ok(size) if size > 0 => return size,
            _ => println!("Please enter a valid positive integer."),
        }
    }
}

fn read_file_name() -> String {
    loop {
        print!("Enter File Name: ");
        io::stdout().flush().unwrap(); // flush to show prompt immediately

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let filename = input.trim();

        if !filename.is_empty() {
            return filename.to_string();
        } else {
            println!("Please enter a valid file name.");
        }
    }
}