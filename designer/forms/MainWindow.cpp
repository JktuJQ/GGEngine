#include "forms/MainWindow.h"


MainWindow::MainWindow(QWidget *parent)
        : QMainWindow(parent) {
    // Ui
    setupUi(this);
    initUi();
    setIcons();
}


void MainWindow::initUi() {
    connect(actionNew_Project, &QAction::triggered, this, &MainWindow::actionCreateNewProject);
    connect(actionExit, &QAction::triggered, this, &QApplication::quit);
}

void MainWindow::setIcons() {

}


void MainWindow::actionCreateNewProject() {
    dialogNewProject = new NewProjectDialog(this);

    connect(dialogNewProject, &QDialog::accepted, this, &MainWindow::createProject);

    dialogNewProject->exec();
}


void MainWindow::createProject() {

}
