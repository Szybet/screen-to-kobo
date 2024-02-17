#include "mainwindow.h"
#include "qthread.h"
#include "ui_mainwindow.h"

#include <QDebug>
#include <QGraphicsPixmapItem>
#include <QTimer>
#include <QFile>
#include <QScreen>
#include <QLocalSocket>

#include "data.h"

#include "einkenums.h"
#include "koboplatformfunctions.h"

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    bytes.clear();

    QThread * thread = new QThread();
    dataObject * data = new dataObject();

    QObject::connect(data, &dataObject::dataSend, this, &MainWindow::dataAdd);
    QObject::connect(data, &dataObject::dataFull, this, &MainWindow::dataFull);

    data->moveToThread(thread);
    QObject::connect(thread, &QThread::started, data, &dataObject::start);
    thread->start();

    qDebug() << "Setting waveform mode";
    KoboPlatformFunctions::setFullScreenRefreshMode(WaveForm_A2);
    KoboPlatformFunctions::setFastScreenRefreshMode(WaveForm_A2);
    KoboPlatformFunctions::setPartialScreenRefreshMode(WaveForm_A2);
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::dataAdd(QByteArray data) {
    qDebug() << "Adding data";
    bytes.append(data);
}

void MainWindow::dataFull() {
    qDebug() << "Data full with length:" << bytes.size();
    refresh();
    bytes.clear();
}

void MainWindow::refresh() {
    qDebug() << "Updating mainwindow...";

    QPixmap pixmap;
    pixmap.loadFromData(bytes, "png", Qt::AutoColor);
    ui->label->setPixmap(pixmap);
}


