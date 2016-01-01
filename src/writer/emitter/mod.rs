//! Contains FBX emitters.

use std::io::{Write, Seek};
use writer::error::{Result, Error};
use writer::{FbxEvent, EmitterConfig};
use common::FbxFormatType;
use self::binary::BinaryEmitter;
use self::ascii::AsciiEmitter;

mod ascii;
mod binary;

#[derive(Debug, Clone)]
enum EmitterState {
    /// Emitter is initialized but not used yet.
    Initial,
    /// Emitting Binary FBX.
    Binary(BinaryEmitter),
    /// Emitting ASCII FBX.
    Ascii(AsciiEmitter),
}

#[derive(Debug, Clone)]
struct CommonState{
    final_result: Option<Result<()>>,
}

pub struct Emitter {
    config: EmitterConfig,
    common: CommonState,
    state: EmitterState,
}

impl Emitter {
    pub fn new(config: EmitterConfig) -> Self {
        Emitter {
            config: config,
            common: CommonState {
                final_result: None,
            },
            state: EmitterState::Initial,
        }
    }

    pub fn write<'a, W: Write + Seek>(&mut self, sink: &mut W, event: FbxEvent<'a>) -> Result<()> {
        if let Some(ref result) = self.common.final_result {
            return result.clone();
        }
        let result = match self.state {
            EmitterState::Initial => {
                match event {
                    FbxEvent::StartFbx(FbxFormatType::Binary(ver)) => {
                        let mut emitter = BinaryEmitter::new(ver);
                        let result = emitter.emit_start_fbx(sink, ver);
                        self.state = EmitterState::Binary(emitter);
                        result
                    },
                    FbxEvent::StartFbx(FbxFormatType::Ascii) => {
                        let mut emitter = AsciiEmitter::new();
                        let result = emitter.emit_start_fbx(sink);
                        self.state = EmitterState::Ascii(emitter);
                        result
                    },
                    _ => {
                        Err(Error::FbxNotStarted)
                    }
                }
            },
            EmitterState::Binary(ref mut emitter) => match event {
                FbxEvent::StartFbx(_) => Err(Error::FbxAlreadyStarted),
                FbxEvent::EndFbx => emitter.emit_end_fbx(sink),
                FbxEvent::StartNode { name, properties } => emitter.emit_start_node(sink, name, &properties),
                FbxEvent::EndNode => emitter.emit_end_node(sink),
                FbxEvent::Comment(_) => if self.config.ignore_minor_errors {
                    warn!("Comment cannot be exported to Binary FBX");
                    Ok(())
                } else {
                    error!("Comment cannot be exported to Binary FBX");
                    Err(Error::UnwritableEvent)
                },
            },
            EmitterState::Ascii(ref mut _emitter) => match event {
                FbxEvent::StartFbx(_) => Err(Error::FbxAlreadyStarted),
                _ => {
                    Err(Error::Unimplemented("Ascii FBX emitter is unimplemented yet".to_string()))
                }
            },
        };
        if let Err(ref err) = result {
            self.common.final_result = Some(Err(err.clone()));
        }
        result
    }
}