/**
 * Misc interop helpers.
**/

#include "windows.h"
#include "winnls.h"
#include "shobjidl.h"
#include "objbase.h"
#include "objidl.h"
#include "shlguid.h"

extern "C" int saveShortcut(
<<<<<<< Updated upstream
    const char *shortcutPath,
    const char *description,
    const char *path,
    const char *args,
    const char *workingDir) {
    char* errStr = NULL;
=======
    const wchar_t *shortcutPath,
    const wchar_t *description,
    const wchar_t *path,
    const wchar_t *args,
    const wchar_t *workingDir,
    const wchar_t *exePath)
{
    char *errStr = NULL;
>>>>>>> Stashed changes
    HRESULT h;
    IShellLink* shellLink = NULL;
    IPersistFile* persistFile = NULL;

#ifdef _WIN64
    wchar_t wName[MAX_PATH+1];
#else
    WORD wName[MAX_PATH+1];
#endif

    int id;

    // Initialize the COM library
    h = CoInitialize(NULL);
    if (FAILED(h)) {
        errStr = "Failed to initialize COM library";
        goto err;
    }

    h = CoCreateInstance( CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER,
            IID_IShellLink, (PVOID*)&shellLink );
    if (FAILED(h)) {
        errStr = "Failed to create IShellLink";
        goto err;
    }

    h = shellLink->QueryInterface(IID_IPersistFile, (PVOID*)&persistFile);
    if (FAILED(h)) {
        errStr = "Failed to get IPersistFile";
        goto err;
    }

    //Append the shortcut name to the folder
    MultiByteToWideChar(CP_UTF8,0,shortcutPath,-1,wName,MAX_PATH);

    // Load the file if it exists, to get the values for anything
    // that we do not set.  Ignore errors, such as if it does not exist.
    h = persistFile->Load(wName, 0);

    // Set the fields for which the application has set a value
    if (description!=NULL)
        shellLink->SetDescription(description);
    if (path!=NULL)
        shellLink->SetPath(path);
<<<<<<< Updated upstream
    if (args!=NULL)
=======
    // default to using the first icon in the exe (usually correct)
    if (exePath != NULL)
        shellLink->SetIconLocation(exePath, 0);
    if (args != NULL)
>>>>>>> Stashed changes
        shellLink->SetArguments(args);
    if (workingDir!=NULL)
        shellLink->SetWorkingDirectory(workingDir);

    //Save the shortcut to disk
    h = persistFile->Save(wName, TRUE);
    if (FAILED(h)) {
        errStr = "Failed to save shortcut";
        goto err;
    }

    persistFile->Release();
    shellLink->Release();
    CoUninitialize();
    return h;

err:
    if (persistFile != NULL)
        persistFile->Release();
    if (shellLink != NULL)
        shellLink->Release();
    CoUninitialize();

    return h;
}

