#include "forms/NewProjectDialog.h"


NewProjectDialog::NewProjectDialog(QWidget *parent)
        : QDialog(parent) {
    // Ui
    setupUi(this);
    initUi();
    setIcons();
}


void NewProjectDialog::initUi() {
    connect(pushButton_GetDirectory, &QPushButton::clicked, this, &NewProjectDialog::chooseDirectory);
    connect(lineEdit_SceneName, &QLineEdit::textEdited, this, &NewProjectDialog::changeSceneName);
}

void NewProjectDialog::setIcons() {
    QIcon iconGetDirectory;
    iconGetDirectory.addFile(QString("data/images/icon_directory.jpg"),
                             QSize(),
                             QIcon::Normal,
                             QIcon::Off);
    pushButton_GetDirectory->setIcon(iconGetDirectory);
    pushButton_GetDirectory->setIconSize(QSize(40, 40));
}


void NewProjectDialog::chooseDirectory() {
    lineEdit_GetDirectory->setText(QFileDialog::getExistingDirectory(this, "Choose Directory",
                                                                     "/home",
                                                                     QFileDialog::ShowDirsOnly
                                                                     | QFileDialog::DontResolveSymlinks));
}

void NewProjectDialog::changeSceneName() {
    auto isValid = [=](const std::string &str) {
        if (!((str[0] >= 'a' && str[0] <= 'z')
              || (str[0] >= 'A' && str[1] <= 'Z')
              || str[0] == '_'))
            return false;
        for (int i = 1; i < str.length(); i++) {
            if (!((str[i] >= 'a' && str[i] <= 'z')
                  || (str[i] >= 'A' && str[i] <= 'Z')
                  || (str[i] >= '0' && str[i] <= '9')
                  || str[i] == '_'))
                return false;
        }
        return true;
    };
    if (lineEdit_SceneName->text().toStdString().length() != 0) {
        if (not isValid(lineEdit_SceneName->text().toStdString())) {
            lineEdit_SceneName->backspace();
        }
    }
}


void NewProjectDialog::accept() {
    std::string projectName = lineEdit_ProjectName->text().toStdString(),
            projectDirectory = lineEdit_GetDirectory->text().toStdString(),
            projectScene = lineEdit_SceneName->text().toStdString();

    if (projectName.length() < 4)
        label_errormessage->setText("Project name must be at least 4 characters long.");
    else if (projectDirectory.length() < 2)
        label_errormessage->setText("Project directory must be chosen.");
    else if (projectScene.length() < 2)
        label_errormessage->setText("Scene name must be at least 2 characters long.");
    else
        QDialog::accept();
}
