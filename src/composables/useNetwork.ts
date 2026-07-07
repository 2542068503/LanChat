import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { peers, peerProfiles, selfInfo, isOnline } from '../store';
import type { Peer } from '../types';

export function useNetwork() {
  const saveProfilesToLocalStorage = () => {
    localStorage.setItem("peerProfiles", JSON.stringify(peerProfiles.value));
  };

  const loadProfilesFromLocalStorage = () => {
    const data = localStorage.getItem("peerProfiles");
    if (data) {
      try {
        peerProfiles.value = JSON.parse(data);
      } catch (e) {
        console.error("Failed to parse peerProfiles", e);
      }
    }
  };

  const fetchSelfInfo = async () => {
    try {
      const info = await invoke<any>("get_self_info");
      selfInfo.value = info;
      
      peerProfiles.value[selfInfo.value.id] = {
        username: selfInfo.value.username,
        avatarId: selfInfo.value.avatarId,
        avatarBase64: selfInfo.value.avatarBase64 || undefined
      };
      peerProfiles.value = { ...peerProfiles.value };
      saveProfilesToLocalStorage();
    } catch (e) {
      console.error("Failed to fetch self info", e);
    }
  };

  const fetchNetworkDetails = async () => {
    try {
      isOnline.value = await invoke<boolean>("is_online");
    } catch (e) {
      console.error("Failed to fetch network details", e);
    }
  };

  const setupNetworkListeners = async () => {
    await listen<Peer[]>("peers-updated", (event) => {
      const list = event.payload;
      
      list.forEach(p => {
        if (p.id !== selfInfo.value.id) {
          if (!peerProfiles.value[p.id] || 
              peerProfiles.value[p.id].username !== p.username || 
              peerProfiles.value[p.id].avatarId !== (p as any).avatarId ||
              peerProfiles.value[p.id].avatarBase64 !== (p as any).avatarBase64) {
            
            peerProfiles.value[p.id] = {
              username: peerProfiles.value[p.id]?.remark || p.username,
              avatarId: (p as any).avatarId || 1,
              avatarBase64: (p as any).avatarBase64 || undefined,
              remark: peerProfiles.value[p.id]?.remark
            };
          }
        }
      });
      
      peerProfiles.value = { ...peerProfiles.value };
      saveProfilesToLocalStorage();

      peers.value = list.filter(p => p.id !== selfInfo.value.id);
    });

    // Start interval
    setInterval(fetchNetworkDetails, 5000);
  };

  return {
    fetchSelfInfo,
    fetchNetworkDetails,
    setupNetworkListeners,
    loadProfilesFromLocalStorage,
    saveProfilesToLocalStorage
  };
}
