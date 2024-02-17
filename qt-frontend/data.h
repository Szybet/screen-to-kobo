#ifndef DATA_OBJECT_H
#define DATA_OBJECT_H

#include <QObject>
#include <QTimer>

class dataObject : public QObject
{
    Q_OBJECT

public:
    dataObject();
    void start();

signals:
    void dataSend(QByteArray data);
    void dataFull();
};

#endif // DATA_OBJECT_H
