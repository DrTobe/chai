port module Main exposing (..)

import Browser
import Html exposing (Html)
import Html.Attributes --exposing (..)
import Html.Events --exposing (..)
import Json.Decode as D

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



-- MODEL


type alias Model =
  { draft : String
  , messages : List String
  }


init : () -> ( Model, Cmd Msg )
init flags =
  ( { draft = "", messages = [] }
  , Cmd.none
  )


-- UPDATE


type Msg
  = DraftChanged String
  | Send
  | Recv String


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

    Recv message ->
      ( { model | messages = model.messages ++ [message] }
      , Cmd.none
      )



-- SUBSCRIPTIONS


-- Subscribe to the `messageReceiver` port to hear about messages coming in
-- from JS. Check out the index.html file to see how this is hooked up to a
-- WebSocket.
--
subscriptions : Model -> Sub Msg
subscriptions _ =
  messageReceiver Recv



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
      , boardView newBoard
      ]

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

type alias BoardState = 
  { fields: List (Maybe (PieceType, Player))
  , en_passant_field: EnPassantFieldInfo
  }

type alias EnPassantFieldInfo =
  { ply: Int
  , skipped: Int
  , target: Int
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
    fields =  List.map (\piece -> Just (piece, White)) initRow
           ++ List.repeat 8 (Just (InitPawn, White))
           ++ List.repeat (4*8) Nothing
           ++ List.repeat 8 (Just (InitPawn, Black))
           ++ List.map (\piece -> Just(piece, Black)) initRow
    enPassant = { ply = 0         -- this is how it is marked as "invalid"
                , skipped = 0xFF  -- in the Rust code. Those nubers are
                , target = 0xFF } -- unsigned there.
  in
    BoardState fields enPassant

type alias GameState =
  { ply : Int
  , fifty_move_rule_last_event : Int
  , board : BoardState
  }

newGame : GameState
newGame =
  { ply = 0
  , fifty_move_rule_last_event = 0
  , board = newBoard
  }

-- CHESS view

boardView : BoardState -> Element Msg
boardView board =
  let
      subfields = \rowNum -> List.take 8 <| List.drop (rowNum*8) board.fields
      subrow = \rowNum -> rowView rowNum <| subfields rowNum
      sevenToZero = List.range 0 7 |> List.reverse
  in
      column [] <| List.map subrow sevenToZero

rowView : Int -> List (Maybe (PieceType, Player)) -> Element Msg
rowView rowNum fields =
  let
      isDarkField = \colNum -> modBy 2 (rowNum + colNum) == 0
      fieldColor = \colNum -> case isDarkField colNum of
        True -> rgb255 100 100 100
        False -> rgb255 250 250 250
      pieceImgSrc = \ptp -> case ptp of
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
