#ifndef DESIGNER_BASEUI_H
#define DESIGNER_BASEUI_H

// Includes
#include <QWidget>
#include <QMainWindow>



namespace ui {

    class Ui {
    protected:
        virtual void setupUi(QWidget *form) {}; // Setups all widgets
        virtual void setupUi(QMainWindow *form) {};

        virtual void retranslateUi(QWidget *form) {};  // Retranslates text on widgets
        virtual void initUi() {};  // Сonnects all slots and signals
        virtual void setIcons() {}; // Sets all icons
    };

}




#endif
