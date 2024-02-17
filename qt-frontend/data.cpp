#include "data.h"

#include "QLocalServer"
#include "qlocalsocket.h"

#include "QFile"

dataObject::dataObject() {}

void dataObject::start() {
    QFile f("/tmp/screenRGB");
    f.remove();

    QLocalServer* server = new QLocalServer(this);

    connect(server, &QLocalServer::newConnection, [=]() {
        QLocalSocket* socket = server->nextPendingConnection();
        connect(socket, &QLocalSocket::readyRead, [=]() {
            int l = socket->bytesAvailable();
            qDebug() << "Received message with length:" << l;
            if(l != 0) {
                emit dataSend(socket->readAll());
            }
        });
        connect(socket, &QLocalSocket::disconnected, [=]() {
            qDebug() << "Client disconnected";
            dataFull();
        });
    });

    if (!server->listen("/tmp/screenRGB")) {
        qDebug() << "Failed to start server";
        return;
    } else {
        qDebug() << "Server started";
    }
}
