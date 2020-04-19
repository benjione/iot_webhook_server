import websocket
import ssl
import time
import json
import Adafruit_MCP4725

SERVER_URL = '127.0.0.1'
SERVER_PORT = "8088"
HEADERS = {"username": "<<username>>",
           "password": "<<password>>",
           "object": "<<device_name>>"}

LAST_ON = 100

PING_INTERVAL = 7.5

# Create a DAC instance.
dac = Adafruit_MCP4725.MCP4725()
dac.set_voltage(0, persist=True)


def on_message(ws, message):
    print("Received message:")
    print(message)
    global LAST_ON
    level = int(message)
    if level == 101:
        level = LAST_ON
    elif level > 0:
        LAST_ON = level
    voltage = (float(level)/1.7) * 100.0  # Between 0 and 1.7 V
    value = (4096 * voltage)/3.3  # Connected to 3.3 V source
    dac.set_voltage(value)


def on_error(ws, error):
    print(error)


def on_close(ws):
    print("### closed ###")


def on_open(ws):
    ws.send(json.dumps(HEADERS))


if __name__ == "__main__":
    while(True):
        websocket.enableTrace(True)
        ws = websocket.WebSocketApp("ws://" + SERVER_URL + ":" + SERVER_PORT + "/ws",
                                    on_message=on_message,
                                    on_error=on_error,
                                    on_close=on_close)
        ws.on_open = on_open
        ws.run_forever(sslopt={"check_hostname": False,
                               "cert_reqs": ssl.CERT_NONE},
                       ping_interval=PING_INTERVAL)
        time.sleep(0.5)
