port module Main exposing (..)

import Browser
import Browser.Events
import Html exposing (Html)
import Html.Attributes --exposing (..)
import Html.Events --exposing (..)
import Json.Decode as D
import Json.Encode as E
import Time

-- elm-ui
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Events as Events
import Element.Font as Font
import Element.Input as Input
import Element.Region as Region
import ElmLogo


-- MAIN


main : Program (Int, Int) Model Msg
main =
  Browser.element
    { init = init
    , view = view
    , update = update
    , subscriptions = subscriptions
    }




-- PORTS


port requestNewgame : () -> Cmd msg
port requestValidmoves : (String, Int) -> Cmd msg
port requestMinimax : String -> Cmd msg
port gamestateReceiver : (String -> msg) -> Sub msg
port validmovesReceiver : (String -> msg) -> Sub msg



-- MODEL

type PlayMode
  = HumanVsAI
  | AIvsAI

type alias Model =
  { playmode : PlayMode
  , gamestate : GameState
  , selectedField : Maybe Int
  , validmoves : List PotentialMove -- empty if not requested yet
  , windowSize : (Int, Int)
  , error : Maybe String
  }


init : (Int, Int) -> ( Model, Cmd Msg )
init windowSizeFlags =
      {-
  let
      json = """[[21, {"ply": 0, "fifty_move_rule_last_event": 0, "board": {"fields": [], "en_passant_field": {"ply": 0, "skipped": 0, "target": 0}}, "finished": "Ongoing"}]]"""
      _ = Debug.log "json" json
      _ = Debug.log "decode" <| D.decodeString potentialMovesDecoder json
  in
      -}
  ( initModel HumanVsAI windowSizeFlags
  , Cmd.none
  )

initModel : PlayMode -> (Int, Int) -> Model
initModel playmode windowSize =
  { playmode = playmode
  , gamestate = newGame
  , selectedField = Nothing
  , validmoves = []
  , windowSize = windowSize
  , error = Nothing
  }


-- UPDATE


type Msg
  = Tick
  | GotNewWindowSize Int Int
  | Click Int
  | RecvGameState (Result D.Error GameState)
  | RecvValidmoves (Result D.Error (List PotentialMove))
  | Restart
  | PlayHuman
  | PlayAI
  | HelpRequested


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
  case msg of
    Restart ->
      ( initModel model.playmode model.windowSize
      , Cmd.none
      )

    PlayHuman ->
      ( { model | playmode = HumanVsAI }
      , Cmd.none
      )

    PlayAI ->
      ( { model | playmode = AIvsAI }
      , Cmd.none
      )

    HelpRequested ->
      ( model
      , if turn model.gamestate == White && model.playmode == HumanVsAI
          then createMinimaxRequest model
          else Cmd.none
      )

    Click field ->
      case List.filterMap (\potMove -> if potMove.new_field == field then Just potMove.new_state else Nothing) model.validmoves |> List.head of
        Just newState -> 
          ( { model | gamestate = newState
                    , selectedField = Nothing
                    , validmoves = []
            }
          , Cmd.none
          )
        Nothing -> 
          case getAtOr model.gamestate.board.fields field Nothing of
            Just { player } ->
              if    player == White
                 && turn model.gamestate == White
                 && model.playmode == HumanVsAI
                then ( { model | selectedField = Just field }
                     , requestValidmoves ((E.encode 0 <| gameStateEncoder model.gamestate), field)
                     )
                else ( { model | selectedField = Nothing, validmoves = [] }
                     , Cmd.none
                     )
            Nothing ->
              ( { model | selectedField = Nothing, validmoves = [] }
              , Cmd.none
              )

    Tick ->
      ( model
      , case model.gamestate.finished of
          Ongoing -> case (turn model.gamestate, model.playmode) of
            (Black, _) -> createMinimaxRequest model
            (White, AIvsAI) -> createMinimaxRequest model
            _ -> Cmd.none
          _ -> Cmd.none
      )

    RecvGameState gamestate ->
      ( { model | gamestate = Result.withDefault newGame gamestate
                , selectedField = Nothing
                , validmoves = []
                , error = updateError model gamestate 
                                  "Could not decode game state from JSON."
        }
      , Cmd.none
      )

    RecvValidmoves validmoves ->
      ( { model | validmoves = Result.withDefault [] validmoves
                , error = updateError model validmoves
                                  "Could not decode valid moves from JSON."
        }
      , Cmd.none
      )

    GotNewWindowSize width height ->
      ( { model | windowSize = (width, height)
        }
      , Cmd.none
      )

updateError : Model -> (Result D.Error x) -> String -> Maybe String
updateError model decodeResult errMsg =
  case decodeResult of
    Ok _ -> model.error
    Err decodeError -> Just <| errMsg ++ "\n\n" ++ D.errorToString decodeError

createMinimaxRequest model =
  requestMinimax <| E.encode 0 <| gameStateEncoder model.gamestate

getAt : List a -> Int -> Maybe a
getAt lst index = List.head <| List.drop index lst

getAtOr : List a -> Int -> a -> a
getAtOr lst index default = Maybe.withDefault default <| getAt lst index

-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
  Sub.batch
    [ Time.every 1000 (\ignore -> Tick)
    , Browser.Events.onResize GotNewWindowSize
    , gamestateReceiver (\json -> D.decodeString gameStateDecoder json |> RecvGameState)
    , validmovesReceiver (\json -> D.decodeString potentialMovesDecoder json |> RecvValidmoves)
    ]



-- VIEW


view : Model -> Html Msg
view model =
  let
      highlightedFields =  List.map (\move -> move.new_field) model.validmoves
                        ++ case model.selectedField of
                              Just field -> [ field ]
                              Nothing -> []
  in
    layout [] <|
      el [ width fill
         , height fill
         ] <|
        column [ width fill
               , height fill
               ]
          [ column [ centerX
                   , centerY
                   ]
              [ buttons model
              , boardView (boardSize model) model.gamestate.board highlightedFields Click
              , winMessage model
              , errorMessage model
              --, text <| String.fromInt (Tuple.first model.windowSize) ++ "x" ++ String.fromInt (Tuple.second model.windowSize)
              --, text (Debug.toString <| classifyDevice { width = Tuple.first model.windowSize, height = Tuple.second model.windowSize })
              ]
          , el [ centerX
               , width <| px (boardSize model)
               ] logos
          ]

boardSize : Model -> Int
boardSize model =
  let
      size = model.windowSize
      width = Tuple.first size
      height = Tuple.second size
      device = classifyDevice { width = width, height = height }
      --_ = Debug.log "device" device
  in
      {-
      case device.class of
        Phone -> min width height - 50
        Tablet -> min width height - 50
        Desktop -> min 400 <| min width height
        BigDesktop -> min 600 <| min width height
      -}
      min (width-20) <| min (height-60) 400

buttons : Model -> Element Msg
buttons model =
  row [ width <| px (boardSize model)
      , paddingXY 0 40
      , spacing 5
      ]
      [ button "Restart" Restart
      --, button "Help!" HelpRequested
      , if model.playmode == AIvsAI
          then button "Take Control" PlayHuman
          else button "Just Watch" PlayAI
      ]

button : String -> Msg -> Element Msg
button btntext msg =
  Input.button
    [ width fill
    , padding 10
    , Border.width 1
    , Border.rounded 3
    , mouseOver [ Border.shadow 
                   { offset = (1,1)
                   , size = 2
                   , blur = 2
                   , color = rgba 0 0 0 0.2
                   }
                ]
    , focused []
    ]
    { onPress = Just msg
    , label = el [ centerX ] <| text btntext
    }

winMessage : Model -> Element Msg
winMessage model =
  let
      gs = model.gamestate
      mover = playerEncoder <| turn gs
      other = playerEncoder <| turn { gs | ply = gs.ply + 1 }
  in
    el [ width <| px (boardSize model)
       , height <| px 50
       ] <|
       el [ centerX
          , centerY
          ] <|
        case model.gamestate.finished of
          Ongoing -> none
          Checkmate -> text <| other ++ " wins!"
          Stalemate -> text <| "Draw! " ++ mover ++ " can not move (Stalemate)."
          ThreefoldRepetition -> text <| "Draw (threefold board repetition)."
          FiftyMoveDraw -> text <| "Draw (fifty move rule)."

errorMessage : Model -> Element Msg
errorMessage model =
  case model.error of
    Nothing -> none
    Just errMsg -> 
      column [ width <| px (boardSize model)
             , paddingXY 0 50
             ]
             [ el [ Font.bold, Font.size 24 ] <| text "Error:"
             , text errMsg
             ]

logos : Element Msg
logos =
  row [ centerX
      , spacing 20
      , paddingXY 0 10
      ]
      [ el [ width <| px 40 ] <| ElmLogo.element 40
      , image [ width <| px 40
              ]
              { src = "logos/rust-logo-blk.svg"
              , description = "The Rust Programming Language Logo"
              }
      , image [ width <| px 40
              ]
              { src = "logos/WebAssembly_Logo.svg"
              , description = "The WebAssembly Logo"
              }
      ]

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

turn : GameState -> Player
turn game =
  if modBy 2 game.ply == 1 then Black else White

-- CHESS serde

potentialMovesDecoder : D.Decoder (List PotentialMove)
potentialMovesDecoder =
  D.list potentialMoveDecoder

potentialMoveDecoder : D.Decoder PotentialMove
potentialMoveDecoder =
  D.map2 PotentialMove
    (D.index 0 D.int)
    (D.index 1 gameStateDecoder)

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

boardView : Int -> BoardState -> List Int -> (Int -> Msg) -> Element Msg
boardView size board highlightedFields msg =
  let
      subfields = \rowNum -> List.take 8 <| List.drop (rowNum*8) board.fields
      subrow = \rowNum -> rowView size rowNum (subfields rowNum) highlightedFields msg
      sevenToZero = List.range 0 7 |> List.reverse
  in
      column [] <| List.map subrow sevenToZero

rowView : Int -> Int -> List (Maybe OccupiedField) -> List Int -> (Int -> Msg) -> Element Msg
rowView size rowNum fields highlightedFields msg =
  let
      fieldSize = size // 8
      fieldNum = \colNum -> rowNum * 8 + colNum
      isDarkField = \colNum -> modBy 2 (rowNum + colNum) == 0
      isHighlighted = \colNum -> List.member (fieldNum colNum) highlightedFields
      fieldColor = \colNum -> case (isDarkField colNum, isHighlighted colNum) of
        (True, False)-> rgb255 100 100 100
        (False, False) -> rgb255 250 250 250
        (True, True) -> rgb255 100 100 0
        (False, True) -> rgb255 200 200 0
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
        image [ width fill
              , height fill
              ] { src = "piece-images/" ++ pieceImgSrc ptp
                , description = pieceImgSrc ptp
                }
      field = \colNum maybePtp ->
        el [ Background.color <| fieldColor colNum
           , width <| px fieldSize
           , height <| px fieldSize
           , Events.onClick <| msg (rowNum*8 + colNum)
           ] <| case maybePtp of
             Just ptp -> pieceImg ptp
             Nothing -> none
  in
      row [] <| List.indexedMap field fields
