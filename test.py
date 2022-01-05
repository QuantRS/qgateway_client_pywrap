import qgateway_client_pywrap

if __name__ == '__main__':
    client = qgateway_client_pywrap.new_client("ws://172.1.0.239:8899", "b703dde0-51da-38ac-a1ea-4e941b9762df")

    def callback(body):
        print(body)
        client.send('6ab1479f-cf4e-39d6-b722-262c56e55e45', "haha", body)
    client.subscribe("6ab1479f-cf4e-39d6-b722-262c56e55e45", ["ahha"], callback)

    from threading import Event
    Event().wait()