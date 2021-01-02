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
