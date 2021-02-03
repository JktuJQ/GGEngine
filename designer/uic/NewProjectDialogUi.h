#ifndef DESIGNER_NEWPROJECTDIALOGUI_H
#define DESIGNER_NEWPROJECTDIALOGUI_H

// Includes
#include "BaseUi.h"

#include <QtCore/QVariant>
#include <QtWidgets/QAction>
#include <QtWidgets/QApplication>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QDialog>
#include <QtWidgets/QDialogButtonBox>
#include <QtWidgets/QFileDialog>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QPushButton>



namespace ui {

    class NewProjectDialogUi : public Ui {
    public:
        // Widgets
        QDialogButtonBox *buttonBox{};
        QLabel *label_ProjectName{};
        QLineEdit *lineEdit_ProjectName{};
        QLabel *label_ProjectDirectory{};
        QPushButton *pushButton_GetDirectory{};
        QLineEdit *lineEdit_GetDirectory{};
        QLabel *label_SceneName{};
        QLineEdit *lineEdit_SceneName{};
        QLabel *label_errormessage{};



        void setupUi(QWidget *form) final;
        void retranslateUi(QWidget *form) final;
    };

}




#endif
