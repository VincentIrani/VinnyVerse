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
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use std::io::{self, BufRead, Write};

use serde::{Serialize, Deserialize};
use serde_json;

// Constant Definition /////////////////////////////////////////////////////////////////////////////////////////////////////////

// Variables/enums Definition //////////////////////////////////////////////////////////////////////////////////////////////////
//Server states
#[derive(Debug)]
enum ServerState {
    Idle,
    WorldRunning,
}

// Types of Commands
#[derive(Debug)]
enum Command {
    LoadWorld,
    SaveWorld,
    GenerateWorld,
    StartWorldLoop,
    StopWorldLoop,
    StartListener,
    StopListener,
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
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let cmd = match line.trim().to_lowercase().as_str() {
                "load_world" => Command::LoadWorld,
                "save_world" => Command::SaveWorld,
                "generate_world" => Command::GenerateWorld,
                "start_world" => Command::StartWorldLoop,
                "stop_world" => Command::StopWorldLoop,
                "quit" => Command::Quit,
                _ => {
                    println!("Unknown command");
                    continue;
                }
            };
            if input_tx.send(cmd).is_err() {
                break; // receiver dropped
            }
        }
    });

    // Initial state
    let mut state = ServerState::Idle;
    
    let (tx, mut rx) = mpsc::unbounded_channel::<UserInput>();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let mut ws_task_handle: Option<tokio::task::JoinHandle<()>> = None;

    // Placeholder for world and critter_layer with dimensions 2x2
    let mut world = vec![vec![0u8; 2]; 2]; 
    let mut critter_layer: Vec<Vec<Cell>> = vec![vec![Cell::empty(); 2]; 2];

    // This is the server loop
    loop {

        // Handle commands from the command channel
        while let Ok(cmd) = cmd_rx.try_recv() {
            if let Command::Quit = cmd {
                println!("Quitting server.");
                return;
            } else if let Command::GenerateWorld = cmd {
                println!("Generating new world...");
                let mut world_size = read_world_size();
                world = utils::generate_world(world_size);
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

                
                utils::the_sun(&mut world);

                println!("{:?}", build_que);

                utils::build_critters(&mut critter_layer, &mut build_que);

                println!("World size: {}x{}", world.len(), world[0].len());
        
                //utils::visualize_world_console(&world);
                utils::visualize_critter_layer(&critter_layer);

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