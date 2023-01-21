#include <filesystem>
#include <iostream>
#include <vector>
using namespace std;
using namespace std::filesystem;
char formatted[30];
string p;
vector<string> vec;
int main() {
  printf("Please enter a relative path to the targeting folder.\n");
  freopen("additional.toml", "a", stdout);
  cin >> p;
  path str(p);
  if (!exists(str))
    return 1;
  directory_entry entry(str); // 文件入口
  if (entry.status().type() == file_type::directory)
    cerr << "Successfully read the directory:" << endl;
  else
    return 2;
  directory_iterator list(str);
  for (auto &it : list) {
    string filename = it.path().filename().generic_string();
    cerr << filename << endl;
    printf("[[event]]\ndescription = \"\"\nparticipant = []\nimage = "
           "[\"/%s/%s\"]\n",
           p.data(), filename.data());
    if (filename.substr(0, 4) == "IMG_") {
      int ymd, hms;
      sscanf(filename.data(), "IMG_%d_%d.jpg", &ymd, &hms);
      int month = (ymd - (ymd / 10000 * 10000) - (ymd % 100)) / 100;
      int minute = (hms - (hms / 10000 * 10000) - (hms % 100)) / 100;
      printf("date = %d-%d-%d %d:%d:%d\n\n", ymd / 10000, month, ymd % 100,
             hms / 10000, minute, hms % 100);
    } else {
      unsigned long long timestamp = 0;
      sscanf(filename.data(), "%*[a-z,_]%llu.jpg", &timestamp);
      // cerr << timestamp << endl;
      time_t cur_t = timestamp / 1000;
      struct tm *t = gmtime(&cur_t);
      t->tm_hour += 8; // 北京时间 UTC+8
      strftime(formatted, sizeof(formatted), "%Y-%m-%d %H:%M:%S", t);
      printf("date = %s\n\n", formatted);
      // cerr << formatted << endl;
    }
  }
  return 0;
}
