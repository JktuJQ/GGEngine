#ifndef DESIGNER_MAINWINDOW_H
#define DESIGNER_MAINWINDOW_H

// Includes
#include "uic/MainWindowUi.h"

#include "NewProjectDialog.h"



class MainWindow final : public QMainWindow, public ui::MainWindowUi {
public:
    explicit MainWindow(QWidget *parent = nullptr);

private:
    NewProjectDialog *dialogNewProject = nullptr;


    void initUi() final;
    void setIcons() final;

private slots:
    void actionCreateNewProject();

    void createProject();
};



#endif
