#pragma once
#include <steam/isteamgamecoordinator.h>
extern "C" {
    uint32 SteamGC_SendMessage(ISteamGameCoordinator* gc, uint32 unMsgType, const void* pubData, uint32 cubData);
    bool SteamGC_IsMessageAvailable(ISteamGameCoordinator* gc, uint32* pcubMsgSize);
    uint32 SteamGC_RetrieveMessage(ISteamGameCoordinator* gc, uint32* punMsgType, void* pubDest, uint32 cubDest, uint32* pcubMsgSize);
}
