def main():
    s = ""
    for _ in range(50000):
        s = s + "x"
    print(len(s))

if __name__ == "__main__":
    main()
