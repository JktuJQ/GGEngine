#ifndef DESIGNER_MAINWINDOWUI_H
#define DESIGNER_MAINWINDOWUI_H

// Includes
#include "BaseUi.h"

#include <QtCore/QVariant>
#include <QtWidgets/QAction>
#include <QtWidgets/QApplication>
#include <QtWidgets/QButtonGroup>
#include <QtWidgets/QCheckBox>
#include <QtWidgets/QFrame>
#include <QtWidgets/QGroupBox>
#include <QtWidgets/QHeaderView>
#include <QtWidgets/QLabel>
#include <QtWidgets/QLineEdit>
#include <QtWidgets/QMainWindow>
#include <QtWidgets/QMenu>
#include <QtWidgets/QMenuBar>
#include <QtWidgets/QPlainTextEdit>
#include <QtWidgets/QPushButton>
#include <QtWidgets/QDesktopWidget>
#include <QtWidgets/QSpinBox>
#include <QtWidgets/QStatusBar>
#include <QtWidgets/QTabWidget>
#include <QtWidgets/QTreeWidget>
#include <QtWidgets/QWidget>



namespace ui {

    class MainWindowUi : public Ui {
    protected:
        // Widgets
        QAction *actionNew_Project{};
        QAction *actionOpen_Project{};
        QAction *actionOpen_Recent{};
        QAction *actionClose_Project{};
        QAction *actionSettings{};
        QAction *actionExit{};
        QAction *actionUndo{};
        QAction *actionRedo{};
        QAction *actionGet_generated_code{};
        QAction *actionGetGeneratedCode{};
        QAction *actionConvert_in_C{};
        QWidget *centralwidget{};
        QTabWidget *tabWidget_Hierarchy{};
        QWidget *hierarchy{};
        QPushButton *pushButton_AddScene{};
        QTreeWidget *treeWidget_Hierarchy{};
        QTabWidget *tabWidget_Inspector{};
        QWidget *inspector{};
        QLabel *label_GameObjectName{};
        QLineEdit *lineEdit_GameObjectName{};
        QLabel *label_GameObjectTag{};
        QLineEdit *lineEdit_GameObjectTag{};
        QGroupBox *groupBox_Components{};
        QPushButton *pushButton_AddComponent{};
        QTreeWidget *treeWidget_Components{};
        QFrame *frame_Preview{};
        QGroupBox *groupBox{};
        QLabel *label_CreateGameObjectTag{};
        QLabel *label_CreateGameObjectName{};
        QLineEdit *lineEdit_CreateGameObjectName{};
        QLineEdit *lineEdit_CreateGameObjectTag{};
        QCheckBox *checkBox_Hitbox{};
        QSpinBox *spinBox_X{};
        QLabel *label_X{};
        QLabel *label_Y{};
        QSpinBox *spinBox_Y{};
        QLabel *label_W{};
        QSpinBox *spinBox_W{};
        QSpinBox *spinBox_H{};
        QLabel *label_H{};
        QCheckBox *checkBox_Sprite{};
        QLabel *label_TextureName{};
        QLineEdit *lineEdit_TextureName{};
        QPlainTextEdit *plainTextEdit{};
        QPushButton *pushButton_CreateGameObject{};
        QLineEdit *lineEdit_{};
        QTabWidget *tabWidget_Terminal{};
        QWidget *tab{};
        QPlainTextEdit *plainTextEdit_Terminal{};
        QLineEdit *lineEdit_Terminal{};
        QPushButton *pushButton_Terminal{};
        QPushButton *pushButton_Play{};
        QPushButton *pushButton_Pause{};
        QMenuBar *menubar{};
        QMenu *menuProject{};
        QMenu *menuEdit{};
        QMenu *menuCode{};
        QStatusBar *statusbar{};


        void setupUi(QMainWindow *MainWindow) final;

        void retranslateUi(QWidget *MainWindow) final;
    };

}



#endif
