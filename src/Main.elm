port module Main exposing (..)

import Browser
import Html exposing (Html)
import Html.Attributes --exposing (..)
import Html.Events --exposing (..)
import Json.Decode as D
import Json.Encode as E

-- elm-ui
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Events as Events
import Element.Font as Font
import Element.Input as Input
import Element.Region as Region


-- MAIN


main : Program () Model Msg
main =
  Browser.element
    { init = init
    , view = view
    , update = update
    , subscriptions = subscriptions
    }




-- PORTS


port sendMessage : String -> Cmd msg
port messageReceiver : (String -> msg) -> Sub msg

port requestMinimax : String -> Cmd msg
port gamestateReceiver : (String -> msg) -> Sub msg



-- MODEL


type alias Model =
  { draft : String
  , messages : List String
  , gamestate : Result D.Error GameState
  }


init : () -> ( Model, Cmd Msg )
init flags =
  ( { draft = "", messages = [], gamestate = Ok newGame }
  , Cmd.none
  )


-- UPDATE


type Msg
  = DraftChanged String
  | Send
  | RecvString String
  | RecvGameState (Result D.Error GameState)


-- Use the `sendMessage` port when someone presses ENTER or clicks
-- the "Send" button. Check out index.html to see the corresponding
-- JS where this is piped into a WebSocket.
--
update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
  case msg of
    DraftChanged draft ->
      ( { model | draft = draft }
      , Cmd.none
      )

    Send ->
      ( { model | draft = "" }
      , sendMessage model.draft
      )

    RecvString message ->
      ( { model | messages = model.messages ++ [message] }
      , Cmd.none
      )

    RecvGameState gamestate ->
      ( { model | gamestate = gamestate }
      , case gamestate of
          Ok gs -> case gs.finished of
            Ongoing -> requestMinimax <| E.encode 0 <| gameStateEncoder gs
            _ -> Cmd.none
          Err gs -> Cmd.none
      )



-- SUBSCRIPTIONS


-- Subscribe to the `messageReceiver` port to hear about messages coming in
-- from JS. Check out the index.html file to see how this is hooked up to a
-- WebSocket.
--
subscriptions : Model -> Sub Msg
subscriptions _ =
  Sub.batch
    [ messageReceiver RecvString
    , gamestateReceiver (\json -> D.decodeString gameStateDecoder json |> RecvGameState)
    ]



-- VIEW


view : Model -> Html Msg
view model =
  layout [] <|
    column [] <|
      [ el [ Region.heading 1 ] <| Element.text "Echo Chat"
      ]
      ++ List.map (\msg -> el [] (text <|  "- " ++ msg )) model.messages ++
      [ Input.text [ onEnter Send ]
          { onChange = DraftChanged
          , text = model.draft
          , placeholder =
              Just
                  (Input.placeholder []
                      (text "type some text here and press enter")
                  )
          , label = Input.labelAbove [] (text "My Text Input")
          }
      , Input.button []
          { onPress = Just Send
          , label = text "Send"
          }
      , maybeBoardView model
      ]

maybeBoardView : Model -> Element Msg
maybeBoardView model =
  case model.gamestate of
    Ok gamestate -> boardView gamestate.board
    Err jsonDecErr -> text <| D.errorToString jsonDecErr

-- DETECT ENTER

onEnter : msg -> Element.Attribute msg
onEnter msg =
    Element.htmlAttribute
        (Html.Events.on "keyup"
            (D.field "key" D.string
                |> D.andThen
                    (\key ->
                        if key == "Enter" then
                            D.succeed msg

                        else
                            D.fail "Not the enter key"
                    )
            )
        )

-- CHESS Model

type PieceType
  = InitKing
  | King
  | Queen
  | InitRook
  | Rook
  | Bishop
  | Knight
  | InitPawn
  | Pawn

type Player
  = Black
  | White

type FinishedState
  = Ongoing -- the player whose turn it is is guaranteed to have at least one legal move!
  | Checkmate
  | Stalemate
  | ThreefoldRepetition -- not implemented yet as of 2020-01-04
  | FiftyMoveDraw

type alias BoardState = 
  { fields: List (Maybe OccupiedField)
  , en_passant_field: EnPassantFieldInfo
  }

type alias OccupiedField =
  { piece_type : PieceType
  , player : Player
  }

type alias EnPassantFieldInfo =
  { ply: Int
  , skipped: Int
  , target: Int
  }

type alias GameState =
  { ply : Int
  , fifty_move_rule_last_event : Int
  , board : BoardState
  , finished : FinishedState
  }

type alias PotentialMove =
  { new_field : Int
  , new_state : GameState
  }

newBoard : BoardState
newBoard = 
  let
    initRow = [ InitRook
              , Knight
              , Bishop
              , Queen
              , InitKing
              , Bishop
              , Knight
              , InitRook
              ]
    fields =  List.map (\piece -> Just (OccupiedField piece White)) initRow
           ++ List.repeat 8 (Just (OccupiedField InitPawn White))
           ++ List.repeat (4*8) Nothing
           ++ List.repeat 8 (Just (OccupiedField InitPawn Black))
           ++ List.map (\piece -> Just(OccupiedField piece Black)) initRow
    enPassant = { ply = 0         -- this is how it is marked as "invalid"
                , skipped = 0xFF  -- in the Rust code. Those nubers are
                , target = 0xFF } -- unsigned there.
  in
    BoardState fields enPassant

newGame : GameState
newGame =
  { ply = 0
  , fifty_move_rule_last_event = 0
  , board = newBoard
  , finished = Ongoing
  }

-- CHESS serde

potentialMovesDecoder : D.Decoder (List PotentialMove)
potentialMovesDecoder =
  D.list potentialMoveDecoder

potentialMoveDecoder : D.Decoder PotentialMove
potentialMoveDecoder =
  D.map2 PotentialMove
    D.int
    gameStateDecoder

gameStateDecoder : D.Decoder GameState
gameStateDecoder =
  D.map4 GameState
    (D.field "ply" D.int)
    (D.field "fifty_move_rule_last_event" D.int)
    (D.field "board" boardStateDecoder)
    (D.field "finished" finishedStateDecoder)

boardStateDecoder : D.Decoder BoardState
boardStateDecoder =
  D.map2 BoardState
    (D.field "fields" fieldsDecoder)
    (D.field "en_passant_field" enPassantFieldDecoder)

fieldsDecoder : D.Decoder (List (Maybe OccupiedField))
fieldsDecoder =
  D.list <| D.nullable occupiedFieldDecoder

occupiedFieldDecoder : D.Decoder OccupiedField
occupiedFieldDecoder =
  D.map2 OccupiedField
    (D.index 0 pieceTypeDecoder)
    (D.index 1 playerDecoder)

pieceTypeDecoder : D.Decoder PieceType
pieceTypeDecoder =
  let
      s = D.succeed
  in
    D.string |> D.andThen (\pieceString -> case pieceString of
        "InitKing" -> s InitKing
        "King" -> s King
        "Queen" -> s Queen
        "InitRook" -> s InitRook
        "Rook" -> s Rook
        "Bishop" -> s Bishop
        "Knight" -> s Knight
        "InitPawn" -> s InitPawn
        "Pawn" -> s Pawn
        _ -> D.fail <| pieceString ++ " is not a valid PieceType."
      )

playerDecoder : D.Decoder Player
playerDecoder =
  D.string |> D.andThen (\playerString -> case playerString of
      "Black" -> D.succeed Black
      "White" -> D.succeed White
      _ -> D.fail <| playerString ++ " is not a valid PlayerType."
    )

enPassantFieldDecoder : D.Decoder EnPassantFieldInfo
enPassantFieldDecoder =
  D.map3 EnPassantFieldInfo
    (D.field "ply" D.int)
    (D.field "skipped" D.int)
    (D.field "target" D.int)

finishedStateDecoder : D.Decoder FinishedState
finishedStateDecoder =
  let
      s = D.succeed
  in
    D.string |> D.andThen (\finString -> case finString of
        "Ongoing" -> s Ongoing
        "Checkmate" -> s Checkmate
        "Stalemate" -> s Stalemate
        "ThreefoldRepetition" -> s ThreefoldRepetition
        "FiftyMoveDraw" -> s FiftyMoveDraw
        _ -> D.fail <| finString ++ " is not a valid FinishedState."
      )


gameStateEncoder : GameState -> E.Value
gameStateEncoder gs =
  E.object
    [ ("ply", E.int gs.ply)
    , ("fifty_move_rule_last_event", E.int gs.fifty_move_rule_last_event)
    , ("board", boardStateEncoder gs.board)
    , ("finished", finishedStateEncoder gs.finished)
    ]

boardStateEncoder : BoardState -> E.Value
boardStateEncoder bs =
  E.object
    [ ("fields", fieldsEncoder bs.fields)
    , ("en_passant_field", enPassantFieldEncoder bs.en_passant_field)
    ]

fieldsEncoder : (List (Maybe OccupiedField)) -> E.Value
fieldsEncoder maybeFieldList =
  E.list maybeOccupiedFieldEncoder maybeFieldList

maybeOccupiedFieldEncoder : Maybe OccupiedField -> E.Value
maybeOccupiedFieldEncoder maybeField =
  case maybeField of
    Just field -> occupiedFieldEncoder field
    Nothing -> E.null

occupiedFieldEncoder : OccupiedField -> E.Value
occupiedFieldEncoder field =
  E.list E.string
    [ pieceTypeEncoder field.piece_type
    , playerEncoder field.player
    ]

pieceTypeEncoder : PieceType -> String
pieceTypeEncoder piece =
  case piece of
        InitKing -> "InitKing" 
        King -> "King" 
        Queen -> "Queen" 
        InitRook -> "InitRook" 
        Rook -> "Rook" 
        Bishop -> "Bishop" 
        Knight -> "Knight" 
        InitPawn -> "InitPawn" 
        Pawn -> "Pawn" 

playerEncoder : Player -> String
playerEncoder player =
  case player of
      Black -> "Black"
      White -> "White"

enPassantFieldEncoder : EnPassantFieldInfo -> E.Value
enPassantFieldEncoder epf =
  E.object
    [ ("ply", E.int epf.ply)
    , ("skipped", E.int epf.ply)
    , ("target", E.int epf.ply)
    ]

finishedStateEncoder : FinishedState -> E.Value
finishedStateEncoder fin =
  E.string (
    case fin of
        Ongoing -> "Ongoing" 
        Checkmate -> "Checkmate" 
        Stalemate -> "Stalemate" 
        ThreefoldRepetition -> "ThreefoldRepetition" 
        FiftyMoveDraw -> "FiftyMoveDraw" 
  )

-- CHESS view

boardView : BoardState -> Element Msg
boardView board =
  let
      subfields = \rowNum -> List.take 8 <| List.drop (rowNum*8) board.fields
      subrow = \rowNum -> rowView rowNum <| subfields rowNum
      sevenToZero = List.range 0 7 |> List.reverse
  in
      column [] <| List.map subrow sevenToZero

rowView : Int -> List (Maybe OccupiedField) -> Element Msg
rowView rowNum fields =
  let
      isDarkField = \colNum -> modBy 2 (rowNum + colNum) == 0
      fieldColor = \colNum -> case isDarkField colNum of
        True -> rgb255 100 100 100
        False -> rgb255 250 250 250
      pieceImgSrc = \ptp -> case (ptp.piece_type, ptp.player) of
        (InitKing, Black) -> "black_king.png"
        (King, Black) -> "black_king.png"
        (Queen, Black) -> "black_queen.png"
        (InitRook, Black) -> "black_rook.png"
        (Rook, Black) -> "black_rook.png"
        (Bishop, Black) -> "black_bishop.png"
        (Knight, Black) -> "black_knight.png"
        (InitPawn, Black) -> "black_pawn.png"
        (Pawn, Black) -> "black_pawn.png"
        (InitKing, White) -> "white_king.png"
        (King, White) -> "white_king.png"
        (Queen, White) -> "white_queen.png"
        (InitRook, White) -> "white_rook.png"
        (Rook, White) -> "white_rook.png"
        (Bishop, White) -> "white_bishop.png"
        (Knight, White) -> "white_knight.png"
        (InitPawn, White) -> "white_pawn.png"
        (Pawn, White) -> "white_pawn.png"
      pieceImg = \ptp ->
        image [ width <| px 40
              , height <| px 40
              ] { src = "piece-images/" ++ pieceImgSrc ptp
                , description = pieceImgSrc ptp
                }
      field = \colNum maybePtp ->
        el [ Background.color <| fieldColor colNum
           , width <| px 40
           , height <| px 40
           ] <| case maybePtp of
             Just ptp -> pieceImg ptp
             Nothing -> none

  in
      row [] <| List.indexedMap field fields
