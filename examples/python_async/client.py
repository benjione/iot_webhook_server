import argparse
import asyncio
import signal
import sys

import aiohttp
import json

queue = asyncio.Queue()

WS_TIMEOUT = 10.0
WS_HEARTBEAT = 5.0
SERVER_URL = '127.0.0.1'
SERVER_PORT = 8088
HEADERS = {"username": "<<username>>",
           "password": "<<password>>",
           "object": "<<device_name>>"}


async def start_client(url, loop):
    print("Function called")
    print(url)

    ws = await aiohttp.ClientSession().ws_connect(url,
                                                  method='GET',
                                                  # timeout=WS_TIMEOUT,
                                                  receive_timeout=10.0,
                                                  heartbeat=WS_HEARTBEAT,
                                                  autoclose=False,
                                                  autoping=False)

    asyncio.ensure_future(queue.put(ws.send_str(json.dumps(HEADERS))))

    async def dispatch():
        while True:
            msg = await ws.receive()
            if msg.type == aiohttp.WSMsgType.TEXT:
                print('Text: ', msg.data.strip())
            elif msg.type == aiohttp.WSMsgType.BINARY:
                print('Binary: ', msg.data)
            elif msg.type == aiohttp.WSMsgType.PING:
                print("Ping received, response with pong.")
                await ws.pong()
            elif msg.type == aiohttp.WSMsgType.PONG:
                print('Pong received')
            else:
                if msg.type == aiohttp.WSMsgType.CLOSE:
                    await ws.close()
                elif msg.type == aiohttp.WSMsgType.ERROR:
                    print('Error during receive %s' % ws.exception())
                elif msg.type == aiohttp.WSMsgType.CLOSED:
                    pass
                break

    await dispatch()


async def tick():
    while True:
        await (await queue.get())


async def main(url, loop):
    await asyncio.wait([start_client(url, loop), tick()])


ARGS = argparse.ArgumentParser(
    description="websocket console client for wssrv.py example.")
ARGS.add_argument(
    '--host', action="store", dest='host',
    default=SERVER_URL, help='Host name')
ARGS.add_argument(
    '--port', action="store", dest='port',
    default=SERVER_PORT, type=int, help='Port number')

if __name__ == '__main__':
    args = ARGS.parse_args()
    if ':' in args.host:
        args.host, port = args.host.split(':', 1)
        args.port = int(port)

    url = 'ws://{}:{}/ws'.format(args.host, args.port)

    loop = asyncio.get_event_loop()
    loop.add_signal_handler(signal.SIGINT, loop.stop)
    asyncio.Task(main(url, loop))
    loop.run_forever()
