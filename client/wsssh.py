#!/bin/env python3

from rel.rel import threading
import websocket
import rel
import readchar


def on_message(ws, message):
    print(message.decode("utf-8"), end="", flush=True)

def on_error(ws, error):
    pass

def on_close(ws, close_status_code, close_msg):
    print("Closing!")
    pass

def on_open(ws):
    pass

def send_key(ws):
    while True:
        c = readchar.readchar()
        ws.send(c)
        

if __name__ == "__main__":
    ws = websocket.WebSocketApp("ws://localhost:8000/ssh/127.0.0.1:22?user=bilbo&password=insecure_password",
                        on_open=on_open,
                        on_message=on_message,
                        on_error=on_error,
                        on_close=on_close)

    t1 = threading.Thread(target=send_key, args=(ws,))
    t1.start()

    ws.run_forever(dispatcher=rel, reconnect=5)  # Set dispatcher to automatic reconnection, 5 second reconnect delay if connection closed unexpectedly
    rel.signal(2, rel.abort)  # Keyboard Interrupt
    rel.dispatch()





