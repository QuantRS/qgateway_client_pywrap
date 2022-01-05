import time
import qgateway_client_pywrap

if __name__ == '__main__':
    client = qgateway_client_pywrap.new_client("ws://192.168.99.254:8899", "b0107179-42b4-39dd-8220-f48a84b4cef7")
    def callback(data):
        print(data)
    client.subscribe('c09c21f8-c29d-3fb3-86a8-39109742c802', ['haha'], callback)

    from threading import Event
    Event().wait()