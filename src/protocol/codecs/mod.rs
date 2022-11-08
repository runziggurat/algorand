//! Codec implementations for the Algorand network protocol.
//
//  +====================================================================+
//  |                          Binary message                            |  <- algomsg codec
//  +====================================================================+
//
//   \______________________________    _________________________________/
//                                  \  /
//                                   \/
//
//  +====================================================================+
//  |                          WebSocket frame                           |  <- websocket codec
//  +----------+------+--------------------------------------------------+
//  | Opcode   | [..] |                       Data                       |       [..] - other WS fields
//  +====================================================================+
//
//                    \______________________    ________________________/
//                                           \  /
//                                            \/
//
//                    +==================================================+
//                    |           Tagged algod message                   |  <- tagmsg codec
//                    +-----+--------------------------------------------+
//                    | Tag |                Data payload                |
//                    +==================================================+
//
//                          \_____________________   ____________________/
//                                                \ /
//                               (three options)   |
//                  +------------------------------+
//                  |
//                  |       +============================================+
//                  +---->  |               MsgPack payload              | <- payload codec [msgpack codec]
//                  |       +============================================+
//                  |
//                  |
//                  |       +============================================+
//                  +---->  |               Topics payload               | <- payload codec [topic codec]
//                  |       +============================================+
//                  |
//                  |
//                  |       +============================================+
//                  +---->  |                  Raw bytes                 | <- payload codec [binary codec]
//                          +============================================+
//

pub mod algomsg;
pub mod msgpack;
pub mod payload;
pub mod tagmsg;
pub mod topic;
pub mod websocket;
