#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QDebug>
#include <QGraphicsPixmapItem>
#include <QTimer>
#include <QFile>
#include <QScreen>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);

    timer = new QTimer(this);
    connect(timer, &QTimer::timeout, this, &MainWindow::refresh);
    timer->setInterval(30);
    timer->start();

    qDebug() << "Setup complete";
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::refresh() {
    QString filePath = "/tmp/ss.png";
    if (QFile::exists(filePath)) {
        QPixmap pixmap(filePath);
        if (!pixmap.isNull()) {
            qDebug() << "Setting pixmap";
            //item->setPixmap(pixmap);
            ui->label->setPixmap(pixmap);
            QFile::remove(filePath);
        } else {
            qDebug() << "Failed to load the image from file. Invalid Pixmap";
        }
    } else {
        //qDebug() << "File does not exist.";
    }
}
