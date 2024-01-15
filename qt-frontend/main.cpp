#include "mainwindow.h"

#include <QApplication>

#include <QGraphicsView>
#include <QGraphicsScene>
#include <QMainWindow>
#include <QScreen>
#include <QDebug>

#include "einkenums.h"
#include "koboplatformfunctions.h"

int main(int argc, char *argv[])
{
    QApplication a(argc, argv);

    KoboPlatformFunctions::setFlashing(false);

    QRect screenGeometry = QGuiApplication::primaryScreen()->geometry();
    int x = screenGeometry.width();
    int y = screenGeometry.height();
    qDebug() << "Screen size:" << x << y;

    MainWindow w;

    w.setAnimated(false);
    w.setFixedSize(x, y);

    QGraphicsScene scene;
    scene.addWidget(&w);

    QGraphicsView customView;

    customView.setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    customView.setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);

    customView.setFixedSize(x, y);

    customView.setViewportUpdateMode(QGraphicsView::MinimalViewportUpdate);
    customView.setOptimizationFlag(QGraphicsView::DontAdjustForAntialiasing);
    customView.setCacheMode(QGraphicsView::CacheBackground);
    customView.setScene(&scene);

    customView.show();

    return a.exec();
}
