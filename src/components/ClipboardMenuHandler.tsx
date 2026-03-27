import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { copyToClipboard } from '../stores/clipboardStore';

export function ClipboardMenuHandler() {
  useEffect(() => {
    const unlisten = Promise.all([
      // 处理菜单复制事件
      listen('menu_copy', async () => {
        try {
          const selection = window.getSelection();
          if (selection && selection.toString()) {
            const selectedText = selection.toString();
            await copyToClipboard(selectedText);
            console.log('Menu copy:', selectedText.substring(0, 50));
          }
        } catch (error) {
          console.error('Menu copy failed:', error);
        }
      }),

      // 处理菜单粘贴事件
      listen('menu_paste', async () => {
        try {
          const text = await readText();
          if (text && document.activeElement) {
            // 如果当前元素是输入框，直接插入文本
            const activeElement = document.activeElement as HTMLInputElement | HTMLTextAreaElement;
            if (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA') {
              const start = activeElement.selectionStart || 0;
              const end = activeElement.selectionEnd || 0;
              const currentValue = activeElement.value;
              const newValue = currentValue.slice(0, start) + text + currentValue.slice(end);
              activeElement.value = newValue;
              activeElement.setSelectionRange(start + text.length, start + text.length);

              // 触发 input 事件，确保 React 能够检测到变化
              const event = new Event('input', { bubbles: true });
              activeElement.dispatchEvent(event);
            } else {
              // 对于其他元素，尝试使用 execCommand
              try {
                document.execCommand('insertText', false, text);
              } catch (e) {
                console.warn('execCommand failed, falling back to clipboard write', e);
              }
            }
            console.log('Menu paste:', text.substring(0, 50));
          }
        } catch (error) {
          console.error('Menu paste failed:', error);
        }
      }),

      // 处理菜单剪切事件
      listen('menu_cut', async () => {
        try {
          const selection = window.getSelection();
          if (selection && selection.toString()) {
            const selectedText = selection.toString();
            await copyToClipboard(selectedText);

            // 尝试删除选中的文本
            if (document.activeElement) {
              const activeElement = document.activeElement as
                | HTMLInputElement
                | HTMLTextAreaElement;
              if (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA') {
                const start = activeElement.selectionStart || 0;
                const end = activeElement.selectionEnd || 0;
                const currentValue = activeElement.value;
                const newValue = currentValue.slice(0, start) + currentValue.slice(end);
                activeElement.value = newValue;
                activeElement.setSelectionRange(start, start);

                // 触发 input 事件
                const event = new Event('input', { bubbles: true });
                activeElement.dispatchEvent(event);
              } else {
                // 对于其他元素，尝试使用 execCommand
                try {
                  document.execCommand('delete');
                } catch (e) {
                  console.warn('execCommand delete failed', e);
                }
              }
            }
            console.log('Menu cut:', selectedText.substring(0, 50));
          }
        } catch (error) {
          console.error('Menu cut failed:', error);
        }
      }),

      // 处理菜单全选事件
      listen('menu_select_all', () => {
        try {
          if (document.activeElement) {
            const activeElement = document.activeElement as HTMLInputElement | HTMLTextAreaElement;
            if (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA') {
              activeElement.select();
            } else {
              // 对于其他元素，选择所有内容
              try {
                document.execCommand('selectAll');
              } catch (e) {
                console.warn('execCommand selectAll failed, using Selection API', e);
                // 备用方案：使用 Selection API
                const range = document.createRange();
                range.selectNodeContents(document.body);
                const selection = window.getSelection();
                selection?.removeAllRanges();
                selection?.addRange(range);
              }
            }
            console.log('Menu select all');
          }
        } catch (error) {
          console.error('Menu select all failed:', error);
        }
      }),
    ]);

    // 清理函数
    return () => {
      unlisten.then((listeners) => {
        listeners.forEach((unlistenFn) => unlistenFn());
      });
    };
  }, []);

  // 这个组件不渲染任何内容
  return null;
}
