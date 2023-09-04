/*****************************************************************************
 *
 * Copyright (c) 1998-2000 Palm Inc. or its subsidiaries.  
 * All rights reserved.
 *
 ****************************************************************************/

/*****************************************************************
 *
 * Conduit Manager API
 *
 ****************************************************************/
#ifndef CONDUIT_MGR_H
#define CONDUIT_MGR_H


#define ERR_CONDUIT_MGR             -1000
#define ERR_INDEX_OUT_OF_RANGE      (ERR_CONDUIT_MGR - 1)
#define ERR_UNABLE_TO_DELETE        (ERR_CONDUIT_MGR - 2)
#define ERR_NO_CONDUIT              (ERR_CONDUIT_MGR - 3)
#define ERR_NO_MEMORY               (ERR_CONDUIT_MGR - 4)
#define ERR_CREATORID_ALREADY_IN_USE (ERR_CONDUIT_MGR - 5)
#define ERR_REGISTRY_ACCESS             (ERR_CONDUIT_MGR - 6)
#define ERR_UNABLE_TO_CREATE_CONDUIT    (ERR_CONDUIT_MGR - 7)
#define ERR_UNABLE_TO_SET_CONDUIT_VALUE (ERR_CONDUIT_MGR - 8)
#define ERR_INVALID_HANDLE              (ERR_CONDUIT_MGR - 9)
#define ERR_BUFFER_TOO_SMALL            (ERR_CONDUIT_MGR - 10)
#define ERR_VALUE_NOT_FOUND             (ERR_CONDUIT_MGR - 11)
#define ERR_INVALID_CREATOR_ID          (ERR_CONDUIT_MGR - 12)
#define ERR_INVALID_POINTER             (ERR_CONDUIT_MGR - 13)
#define ERR_UNABLE_TO_INSTALL_OLD       (ERR_CONDUIT_MGR - 14)
#define ERR_INVALID_CONDUIT_TYPE        (ERR_CONDUIT_MGR - 15)
#define ERR_INVALID_COM_PORT_TYPE       (ERR_CONDUIT_MGR - 16)
#define ERR_NO_LONGER_SUPPORTED			(ERR_CONDUIT_MGR - 17)

//ravi' additions for NotifierManager and InstallConduit manager
#define ERR_INVALID_PATH			(ERR_CONDUIT_MGR - 18)
#define ERR_ALREADY_INSTALLED		(ERR_CONDUIT_MGR - 19)
#define ERR_STORAGE_ACCESS			(ERR_CONDUIT_MGR - 20)
#define ERR_NOTIFIER_NOT_FOUND		(ERR_CONDUIT_MGR - 21)
#define ERR_INSTALL_ID_ALREADY_IN_USE	(ERR_CONDUIT_MGR - 22)
#define ERR_INVALID_INSTALL_ID		(ERR_CONDUIT_MGR - 23)






#define INVALID_CREATORID           0
#define INVALID_PRIORITY            2
#define INVALID_INTEGRATE           0

#define CONDUIT_COMPONENT           0
#define CONDUIT_APPLICATION         1
#define CONDUIT_CONDUITS            10

#define BACKUP_CONDUIT              0
#define MAX_SPECIAL                 BACKUP_CONDUIT

#define DIRECT_COM_PORT             0
#define MODEM_COM_PORT              1
#define MAX_COM_PORT                MODEM_COM_PORT



#define CM_CREATOR_ID_SIZE 8		// Rounded up from 4+1

typedef struct {
    char            szCreatorID[CM_CREATOR_ID_SIZE]; // only need 4 + 1 string terminator, by lets
    int             iReserved;
} CM_CREATORLIST_ITEM_TYPE;

typedef CM_CREATORLIST_ITEM_TYPE *CM_CREATORLIST_TYPE;
typedef struct {
    int             iStructureVersion;
    int             iStructureSize;
    int             iType; // Types CONDUIT_X
    char            szCreatorID[CM_CREATOR_ID_SIZE]; // only need 4 + 1 string terminator, by lets
                                    // make it even.
    DWORD           dwPriority;
    int             iConduitNameOffset;
    int             iDirectoryOffset;
    int             iFileOffset;
    int             iRemoteDBOffset;
    int             iUsernameOffset;
    int             iTitleOffset;
    int             iInfoOffset;
} CmConduitType;

//InstallConduit structure
#ifndef FILEINSTALLTYPE_DEF
#define FILEINSTALLTYPE_DEF

typedef struct
{
	TCHAR		szDir[ 64 ];
	TCHAR		szExt[ 256];
	DWORD		dwMask;
	TCHAR		szModule[ 256 ];
    DWORD       dwCreatorID;
	TCHAR		szName[ 256 ];
} FileInstallType;

#define FILEINSTALLTYPE_SIZE sizeof(FileInstallType)
#endif



#define CM_MIN_CONDUITTYPE_SIZE            sizeof(CmConduitType)
#define CM_CONDUIT_BUFFER_OFFSET           sizeof(CmConduitType) 

#define CONDUIT_VERSION                     100

//	API functions
#define CM_INITIAL_LIB_VERSION 0x0001
#define CM_UPDATE_1            0x0002

WORD WINAPI CmGetLibVersion();

//	Utilities for manipulating Creator ID's
int WINAPI CmConvertCreatorIDToString(DWORD dwID, TCHAR * pString, int *piSize);
int WINAPI CmConvertStringToCreatorID(const TCHAR * pString, DWORD *pdwID);

//
//	Functions for reading current conduit configuration
//
int WINAPI CmGetConduitCount(void);

int WINAPI CmGetCreatorIDList(CM_CREATORLIST_TYPE pCreatorList, int *piSize);
int WINAPI CmGetConduitCreatorID( int iIndex, char *pCreatorID, int *piSize);

int WINAPI CmGetConduitByCreator(    const char *pCreatorID, HANDLE *hStruct);
	
//
//	Functions for installing a new conduit
//
int WINAPI CmInstallConduit( HANDLE hStruct);
	//	Only defines Creator ID for a new conduit; other values must
	//	be set separately.
int WINAPI CmInstallCreator( const char *pCreator, int iType);

//
//	Functions for removing a conduit
//
int WINAPI CmRemoveConduitByCreatorID( const char *pCreatorID);

//
// Creator ID based functions for accessing individual data items.
//
int WINAPI CmSetCreatorPriority(     const char *pCreatorID, DWORD dwPriority);
int WINAPI CmGetCreatorPriority(     const char *pCreatorID, DWORD * pdwPriority);

int WINAPI CmSetCreatorName(         const char *pCreatorID, const TCHAR * pConduitName);
int WINAPI CmGetCreatorName(         const char *pCreatorID, TCHAR * pConduitName, int *piSize);

int WINAPI CmSetCreatorDirectory(    const char *pCreatorID, const TCHAR * pDirectory);
int WINAPI CmGetCreatorDirectory(    const char *pCreatorID, TCHAR * pDirectory, int *piSize);

int WINAPI CmSetCreatorRemote(       const char *pCreatorID, const TCHAR * pRemoteDB);
int WINAPI CmGetCreatorRemote(       const char *pCreatorID, TCHAR * pRemoteDB, int *piSize);

int WINAPI CmSetCreatorUser(         const char *pCreatorID, const TCHAR * pUsername);
int WINAPI CmGetCreatorUser(         const char *pCreatorID, TCHAR * pUsername, int *piSize);

int WINAPI CmSetCreatorTitle(        const char *pCreatorID, const TCHAR * pTitle);
int WINAPI CmGetCreatorTitle(        const char *pCreatorID, TCHAR * pTitle, int *piSize);

int WINAPI CmSetCreatorInfo(         const char *pCreatorID, const TCHAR * pInfo);
int WINAPI CmGetCreatorInfo(         const char *pCreatorID, TCHAR * pInfo, int *piSize);

int WINAPI CmSetCreatorFile(         const char *pCreatorID, const TCHAR * pFile);
int WINAPI CmGetCreatorFile(         const char *pCreatorID, TCHAR * pFile, int *piSize);

// Conduit Type - TO BE OBSOLETED
int WINAPI CmGetCreatorType( const char *pCreator);	// Generally for internal use.

//
//	Functions for integrating applications with PalmPilot Desktop.
//
//	Not guaranteed to be supported in future revisions.
int WINAPI CmSetCreatorIntegrate(    const char *pCreatorID, DWORD dwIntegrate);
int WINAPI CmGetCreatorIntegrate(    const char *pCreatorID, DWORD * pdwIntegrate);

int WINAPI CmSetCreatorModule(       const char *pCreatorID, const TCHAR * pModule);
int WINAPI CmGetCreatorModule(       const char *pCreatorID, TCHAR * pModule, int *piSize);

int WINAPI CmSetCreatorArgument(     const char *pCreatorID, const TCHAR * pArgument);
int WINAPI CmGetCreatorArgument(     const char *pCreatorID, TCHAR * pArgument, int *piSize);

//
//	Functions for accessing other HotSync configuration info.
//
// Port access
int WINAPI CmSetComPort(int iType, const TCHAR *pPort);
int WINAPI CmGetComPort(int iType, TCHAR *pPort, int *piSize);

// Backup conduit
int WINAPI CmSetBackupConduit(const TCHAR *pConduit);
int WINAPI CmGetBackupConduit(TCHAR *pConduit, int *piSize);

// Notifiers
int WINAPI CmSetNotifierDll(int iIndex, const TCHAR *pNotifier);
int WINAPI CmGetNotifierDll(int iIndex, TCHAR *pNotifier, int *piSize);

// PC ident
int WINAPI CmSetPCIdentifier(DWORD dwPCID);
int WINAPI CmGetPCIdentifier(DWORD *pdwPCID);

// Core path
int WINAPI CmGetCorePath(TCHAR *pPath, int *piSize);
int WINAPI CmSetCorePath(const TCHAR *pPath);

// HotSync Path
int WINAPI CmGetHotSyncExecPath(TCHAR *pPath, int *piSize);
int WINAPI CmSetHotSyncExecPath(const TCHAR *pPath);


int WINAPI CmSetCreatorValueDword(const char *pCreatorID, TCHAR * pValue, DWORD dwValue);
int WINAPI CmGetCreatorValueDword(const char *pCreatorID, 
                                  TCHAR * pValue, 
                                  DWORD *dwValue,
                                  DWORD dwDefault);
int WINAPI CmSetCreatorValueString(const char *pCreatorID, TCHAR * pValue, TCHAR * pString);
int WINAPI CmGetCreatorValueString(const char *pCreatorID, 
                                   TCHAR * pValue, 
                                   TCHAR * pString, 
                                   int *piSize,
                                   TCHAR *pDefault);



// Registry Restore Function
int WINAPI CmRestoreHotSyncSettings(BOOL bToDefaults);


// general Palm information storage calls
long WINAPI PiSetValueDword(const char *pFolder, const char *pKey, DWORD dwValue );
long WINAPI PiGetValueDword(const char *pFolder, const char *pKey, DWORD *pdwValue, DWORD dwDefault );
long WINAPI PiSetValueString(const char *pFolder, const char *pKey, const char *pValue);
long WINAPI PiGetValueString(const char *pFolder, const char *pKey, char *pValue, int *piLen, const char *pDefault);
long WINAPI PiSetHotsyncValueDword(const char *pFolder, const char *pKey, DWORD dwValue);
long WINAPI PiGetHotsyncValueDword(const char *pFolder, const char *pKey, DWORD *pdwValue, DWORD dwDefault);
long WINAPI PiSetHotsyncValueString(const char *pFolder, const char *pKey, const char *pValue);
long WINAPI PiGetHotsyncValueString(const char *pFolder, const char *pKey, char *pValue, int *piLen, const char *pDefault);


// Install Conduit API
int WINAPI  ImRegister (const FileInstallType sFIT);
int  WINAPI ImRegisterID(DWORD dwCreatorID);
int  WINAPI ImUnregisterID(DWORD dwCreatorID);
int  WINAPI ImSetDirectory(DWORD dwID, const TCHAR* pDirectory);
int  WINAPI ImSetExtension(DWORD dwID, const TCHAR* pExtension);
int  WINAPI ImSetMask(DWORD dwID, DWORD dwMask);
int  WINAPI ImSetModule(DWORD dwID, const TCHAR* pModule);
int  WINAPI ImSetName(DWORD dwID, const TCHAR* pConduitName);
int  WINAPI ImSetDWord(DWORD dwID, const TCHAR* pValue, DWORD dwValue);
int  WINAPI ImSetString(DWORD dwID, const TCHAR* pValue, TCHAR* pString);
int  WINAPI ImGetDirectory(DWORD dwID, TCHAR* pDirectory, int *piSize);
int  WINAPI ImGetExtension(DWORD dwID, TCHAR* pExtension, int* piSize);
int  WINAPI ImGetMask(DWORD dwID, DWORD* pdwMask);
int  WINAPI ImGetModule(DWORD dwID, TCHAR* pModule, int* piSize);
int  WINAPI ImGetName(DWORD dwID, TCHAR* pConduitName, int* piSize);
int  WINAPI ImGetDWord(DWORD dwID, const TCHAR* pValue, DWORD* pdwMask, DWORD dwDefault);
int  WINAPI ImGetString(DWORD dwID, const TCHAR* pValue, TCHAR *pString, int* piSize, const TCHAR* pDefault);

//Install  Notifier Manager API
int  WINAPI NmRegister(const TCHAR* pNotifierPath);
int  WINAPI NmUnregister(const TCHAR* pNotifierPath);
int  WINAPI NmGetCount(DWORD* pdwCount);
int  WINAPI NmFind(const TCHAR *pNotifier);
int  WINAPI NmGetByIndex(int iIndex, TCHAR *pNotifier, int *piSize);
int  WINAPI NmRenameByIndex(int iIndex, const TCHAR *pNotifier);



#endif