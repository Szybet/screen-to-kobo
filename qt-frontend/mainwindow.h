#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QGraphicsScene>

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();
    void refresh();
    void dataAdd(QByteArray data);
    void dataFull();
    QByteArray bytes;
private:
    Ui::MainWindow *ui;
};
#endif // MAINWINDOW_H
