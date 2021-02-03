#include "NewProjectDialogUi.h"


void ui::NewProjectDialogUi::setupUi(QWidget *form) {
    auto *Dialog_NewProject = dynamic_cast<QDialog *>(form);
    if (Dialog_NewProject->objectName().isEmpty())
        Dialog_NewProject->setObjectName(QString("Dialog_NewProject"));
    Dialog_NewProject->resize(400, 190);
    Dialog_NewProject->setMinimumSize(QSize(400, 190));
    Dialog_NewProject->setMaximumSize(QSize(400, 190));
    buttonBox = new QDialogButtonBox(Dialog_NewProject);
    buttonBox->setObjectName(QString("buttonBox"));
    buttonBox->setGeometry(QRect(10, 160, 381, 32));
    buttonBox->setOrientation(Qt::Horizontal);
    buttonBox->setStandardButtons(QDialogButtonBox::Cancel | QDialogButtonBox::Ok);
    label_ProjectName = new QLabel(Dialog_NewProject);
    label_ProjectName->setObjectName(QString("label_ProjectName"));
    label_ProjectName->setGeometry(QRect(10, 30, 111, 21));
    QFont font;
    font.setPointSize(12);
    label_ProjectName->setFont(font);
    lineEdit_ProjectName = new QLineEdit(Dialog_NewProject);
    lineEdit_ProjectName->setObjectName(QString("lineEdit_ProjectName"));
    lineEdit_ProjectName->setGeometry(QRect(120, 30, 271, 21));
    label_ProjectDirectory = new QLabel(Dialog_NewProject);
    label_ProjectDirectory->setObjectName(QString("label_ProjectDirectory"));
    label_ProjectDirectory->setGeometry(QRect(10, 61, 131, 20));
    label_ProjectDirectory->setFont(font);
    pushButton_GetDirectory = new QPushButton(Dialog_NewProject);
    pushButton_GetDirectory->setObjectName(QString("pushButton_GetDirectory"));
    pushButton_GetDirectory->setGeometry(QRect(360, 60, 31, 23));
    lineEdit_GetDirectory = new QLineEdit(Dialog_NewProject);
    lineEdit_GetDirectory->setObjectName(QString("lineEdit_GetDirectory"));
    lineEdit_GetDirectory->setGeometry(QRect(140, 61, 221, 21));
    lineEdit_GetDirectory->setAlignment(Qt::AlignCenter);
    lineEdit_GetDirectory->setReadOnly(true);
    label_SceneName = new QLabel(Dialog_NewProject);
    label_SceneName->setObjectName(QString("label_SceneName"));
    label_SceneName->setGeometry(QRect(10, 120, 101, 21));
    label_SceneName->setFont(font);
    lineEdit_SceneName = new QLineEdit(Dialog_NewProject);
    lineEdit_SceneName->setObjectName(QString("lineEdit_SceneName"));
    lineEdit_SceneName->setGeometry(QRect(110, 120, 281, 21));
    label_errormessage = new QLabel(Dialog_NewProject);
    label_errormessage->setObjectName(QString("label_errormessage"));
    label_errormessage->setGeometry(QRect(0, 170, 221, 16));
    QFont font1;
    font1.setPointSize(7);
    label_errormessage->setFont(font1);
    label_errormessage->setAlignment(Qt::AlignCenter);
    label_errormessage->raise();
    buttonBox->raise();
    label_ProjectName->raise();
    lineEdit_ProjectName->raise();
    label_ProjectDirectory->raise();
    pushButton_GetDirectory->raise();
    lineEdit_GetDirectory->raise();
    label_SceneName->raise();
    lineEdit_SceneName->raise();

    retranslateUi(Dialog_NewProject);
    QObject::connect(buttonBox, SIGNAL(accepted()), Dialog_NewProject, SLOT(accept()));
    QObject::connect(buttonBox, SIGNAL(rejected()), Dialog_NewProject, SLOT(reject()));

    QMetaObject::connectSlotsByName(Dialog_NewProject);
}

void ui::NewProjectDialogUi::retranslateUi(QWidget *form) {
    auto *Dialog_NewProject = dynamic_cast<QDialog *>(form);
    Dialog_NewProject->setWindowTitle(QApplication::translate("Dialog_NewProject", "New Project", Q_NULLPTR));
    label_ProjectName->setText(QApplication::translate("Dialog_NewProject", "Project name - ", Q_NULLPTR));
    label_ProjectDirectory->setText(QApplication::translate("Dialog_NewProject", "Project directory -", Q_NULLPTR));
    pushButton_GetDirectory->setText(QString());
    lineEdit_GetDirectory->setText(QString());
    lineEdit_GetDirectory->setPlaceholderText(
            QApplication::translate("Dialog_NewProject", "Project directory", Q_NULLPTR));
    label_SceneName->setText(QApplication::translate("Dialog_NewProject", "Scene name -", Q_NULLPTR));
    label_errormessage->setText(QString());
}
