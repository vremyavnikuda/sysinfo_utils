#include <windows.h>
#include <stdio.h>
#include <wbemidl.h>

#pragma comment(lib, "wbemuuid.lib")

int main()
{
    HRESULT hres;

    // Инициализация COM
    hres = CoInitializeEx(0, COINIT_MULTITHREADED);
    if (FAILED(hres))
    {
        printf("Ошибка инициализации COM: 0x%08lx\n", hres);
        return 1;
    }

    // Установка уровня безопасности COM
    hres = CoInitializeSecurity(
        NULL, -1, NULL, NULL, RPC_C_AUTHN_LEVEL_DEFAULT,
        RPC_C_IMP_LEVEL_IMPERSONATE, NULL, EOAC_NONE, NULL);
    if (FAILED(hres))
    {
        printf("Error setting COM security level: 0x%08lx\n", hres);
        CoUninitialize();
        return 1;
    }

    // Подключение к WMI
    IWbemLocator *pLoc = NULL;
    hres = CoCreateInstance(
        &CLSID_WbemLocator, 0, CLSCTX_INPROC_SERVER,
        &IID_IWbemLocator, (LPVOID *)&pLoc);
    if (FAILED(hres))
    {
        printf("Error creating IWbemLocator: 0x%08lx\n", hres);
        CoUninitialize();
        return 1;
    }

    IWbemServices *pSvc = NULL;
    hres = pLoc->lpVtbl->ConnectServer(
        pLoc, L"ROOT\\CIMV2", NULL, NULL, 0, 0, 0, 0, &pSvc);
    if (FAILED(hres))
    {
        printf("Error connecting to WMI: 0x%08lx\n", hres);
        pLoc->lpVtbl->Release(pLoc);
        CoUninitialize();
        return 1;
    }else{
        printf("Connected to WMI successfully.\n");
    }

    // Установка уровня безопасности для прокси
    hres = CoSetProxyBlanket(
        (IUnknown *)pSvc, RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE, NULL,
        RPC_C_AUTHN_LEVEL_CALL, RPC_C_IMP_LEVEL_IMPERSONATE, NULL, EOAC_NONE);
    if (FAILED(hres))
    {
        printf("Error setting proxy: 0x%08lx\n", hres);
        pSvc->lpVtbl->Release(pSvc);
        pLoc->lpVtbl->Release(pLoc);
        CoUninitialize();
        return 1;
    }else{
        printf("Proxy set successfully.\n");
    }

    // Выполнение запроса к классу Win32_VideoController
    IEnumWbemClassObject *pEnumerator = NULL;
    hres = pSvc->lpVtbl->ExecQuery(
        pSvc, L"WQL", L"SELECT Name, AdapterRAM, DriverVersion FROM Win32_VideoController",
        WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY, NULL, &pEnumerator);
    if (FAILED(hres))
    {
        printf("Error executing query: 0x%08lx\n", hres);
        pSvc->lpVtbl->Release(pSvc);
        pLoc->lpVtbl->Release(pLoc);
        CoUninitialize();
        return 1;
    }else{
        printf("Query executed successfully.\n");
    }

    // Получение результатов
    IWbemClassObject *pclsObj = NULL;
    ULONG uReturn = 0;
    while (pEnumerator)
    {
        hres = pEnumerator->lpVtbl->Next(pEnumerator, WBEM_INFINITE, 1, &pclsObj, &uReturn);
        if (uReturn == 0){
            break;
        }else{
            printf("Processing GPU information...\n");
        }

        VARIANT vtProp;

        // Получение имени GPU
        hres = pclsObj->lpVtbl->Get(pclsObj, L"Name", 0, &vtProp, 0, 0);
        if (SUCCEEDED(hres) && vtProp.vt == VT_BSTR)
        {
            char name[256];
            WideCharToMultiByte(CP_ACP, 0, vtProp.bstrVal, -1, name, sizeof(name), NULL, NULL);
            printf("GPU name: %s\n", name);
            VariantClear(&vtProp);
        }else{
            printf("GPU name: Not available (HRESULT: 0x%08lx , Type: %d)\n", hres, vtProp.vt);
        }

        // Получение объема видеопамяти (в байтах)
        hres = pclsObj->lpVtbl->Get(pclsObj, L"AdapterRAM", 0, &vtProp, 0, 0);
        if (SUCCEEDED(hres) && vtProp.vt == VT_UI4)
        {
            printf("Video memory: %u MB\n", vtProp.ulVal / (1024 * 1024));
            VariantClear(&vtProp);
        }else{
            printf("Video memory: Not available (HRESULT: 0x%08lx , Type: %d)\n", hres, vtProp.vt);
        }

        // Получение версии драйвера
        hres = pclsObj->lpVtbl->Get(pclsObj, L"DriverVersion", 0, &vtProp, 0, 0);
        if (SUCCEEDED(hres) && vtProp.vt == VT_BSTR)
        {
            char driver[256];
            WideCharToMultiByte(CP_ACP, 0, vtProp.bstrVal, -1, driver, sizeof(driver), NULL, NULL);
            printf("Driver version: %s\n", driver);
            VariantClear(&vtProp);
        }else{
            printf("Driver version: Not available (HRESULT: 0x%08lx , Type: %d)\n", hres, vtProp.vt);
        }

        pclsObj->lpVtbl->Release(pclsObj);
    }

    // Очистка
    pEnumerator->lpVtbl->Release(pEnumerator);
    pSvc->lpVtbl->Release(pSvc);
    pLoc->lpVtbl->Release(pLoc);
    CoUninitialize();

    printf("DONE.\n");
    return 0;
}