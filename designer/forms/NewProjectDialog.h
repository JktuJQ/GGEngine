#ifndef DESIGNER_NEWPROJECTDIALOG_H
#define DESIGNER_NEWPROJECTDIALOG_H

// Includes
#include "uic/NewProjectDialogUi.h"



class NewProjectDialog final : public QDialog, public ui::NewProjectDialogUi {
public:
    explicit NewProjectDialog(QWidget *parent = nullptr);

private:
    void initUi() final;
    void setIcons() final;

private slots:
    void accept() final;

    void chooseDirectory();
    void changeSceneName();
};


#endif
